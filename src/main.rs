use std::{
    borrow::Cow, collections::HashMap, collections::HashSet, convert::Infallible, fmt::Display,
    str::FromStr, sync::Arc,
};

use futures::future::join_all;
use lexical;
use log::{debug, error, info, warn};
use pretty_env_logger;
use teloxide::{ApiErrorKind, KnownApiErrorKind, utils::command::BotCommand};
use teloxide::{prelude::*, types::*};
use tokio;
use url::Url;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "react to a message", rename = "r")]
    React,
    #[command(description = "show who reacted to a message", rename = "s")]
    Show,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Reaction {
    Laugh,
    Anger,
    Love,
    Up,
    Down,
    Sad,
    Custom(char),
}

static REACTIONS: [Reaction; 6] = [
    Reaction::Love,
    Reaction::Laugh,
    Reaction::Anger,
    Reaction::Sad,
    Reaction::Up,
    Reaction::Down,
];

impl FromStr for Reaction {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "laugh" => Reaction::Laugh,
            "anger" => Reaction::Anger,
            "love" => Reaction::Love,
            "up" => Reaction::Up,
            "down" => Reaction::Down,
            "sad" => Reaction::Sad,
            _ => unimplemented!(),
        })
    }
}

impl Display for Reaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reaction::Laugh => write!(f, "laugh"),
            Reaction::Anger => write!(f, "anger"),
            Reaction::Love => write!(f, "love"),
            Reaction::Up => write!(f, "up"),
            Reaction::Down => write!(f, "down"),
            Reaction::Sad => write!(f, "sad"),
            Reaction::Custom(_) => unimplemented!(),
        }
    }
}

impl Reaction {
    pub fn get_emoji(&self) -> &str {
        match self {
            Reaction::Love => "❤️",
            Reaction::Laugh => "😂",
            Reaction::Anger => "😡",
            Reaction::Sad => "😭",
            Reaction::Up => "👍",
            Reaction::Down => "👎",
            _ => unimplemented!(),
        }
    }
}

type UserId = i32;

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();
    info!("Starting Reactions bot");

    let bot = Bot::from_env();

    Dispatcher::new(bot)
        .messages_handler(move |rx: DispatcherHandlerRx<Message>| {
            rx.for_each_concurrent(None, move |update| async move {
                if let Some(text) = update.update.text() {
                    if let Ok(cmd) = Command::parse(text, "reaxnbot") {
                        if let Err(err) = handle_command(update, cmd).await {
                            warn!("message response failed: {:?}", err);
                        }
                    }
                }
            })
        })
        .callback_queries_handler(move |rx: DispatcherHandlerRx<CallbackQuery>| {
            rx.for_each_concurrent(None, move |update| async move {
                if let Err(err) = handle_query(update).await {
                    warn!("callback query response failed: {:?}", err);
                }
            })
        })
        .dispatch()
        .await;
}

async fn handle_command(cx: UpdateWithCx<Message>, command: Command) -> ResponseResult<()> {
    match command {
        Command::Help => {
            cx.answer(Command::descriptions()).send().await?;
        }
        Command::React => {
            let reply_to_message = {
                match cx.update.reply_to_message() {
                    Some(message) => message,
                    None => {
                        cx.reply_to("You need to reply to a message with /r in order for me to know which message you're reacting to.").send().await?;
                        return Ok(());
                    }
                }
            };

            // fail gracefully if we don't have permissions to delete /r messages
            match cx.delete_message().send().await {
                Ok(_) => {}
                Err(RequestError::ApiError{ kind, .. }) 
                    if kind == ApiErrorKind::Known(KnownApiErrorKind::MessageCantBeDeleted) => {},
                Err(err) => return Err(err),
            };

            cx.answer("\u{034f}")
                .reply_to_message_id(reply_to_message.id)
                .reply_markup(create_markup())
                .disable_web_page_preview(true)
                .send()
                .await?;
        }
        Command::Show => {
            let reply_to_message = {
                match cx.update.reply_to_message() {
                    Some(message) => message,
                    None => {
                        cx.reply_to("You need to reply to a message with /s in order for me to know which message you're asking about.").send().await?;
                        return Ok(());
                    }
                }
            };

            let is_reaction_message = 
                reply_to_message.from().map_or(false, |user| user.is_bot && (user.username).as_deref().unwrap() == "reaxnbot") &&
                reply_to_message.text().map_or(false, |text| text.contains("\u{034f}"));

            if !is_reaction_message {
                cx.reply_to("You can only use /s to reply to a reaction message.").send().await?;
                return Ok(());
            }

            let reactions_users = get_reactions_users(&reply_to_message).unwrap();
            let chat_id = reply_to_message.chat_id();
            let reply_to_message_id = reply_to_message.id;

            // put the context into a temporary Arc so that we can use it in futures
            let cx = Arc::new(cx);

            let text =
                // get the user names associated with each reaction, format and join
                join_all(reactions_users.into_iter().map(|(reaction, users)| {
                    let cx = cx.clone();

                    async move {
                        // get the user names associated with this reaction
                        let user_names = join_all(users.iter().map(|&user_id| {
                            let cx = cx.clone();
                            
                            async move {
                                match cx.bot.get_chat_member(chat_id, user_id).send().await {
                                    Ok(user) => user.user.full_name(),
                                    Err(_) => "(unknown)".to_owned(),
                                }
                            }
                        }))
                        .await;

                        format!("{} — {}", reaction.get_emoji(), user_names.join(", "))
                    }
                }))
                .await
                .join("\n");

            // all of the futures should have been completed at this point (b/c of the joins)
            // so it should be safe to do Arc::try_unwrap(cx), but there is actually no reason to

            // respond to the user

            if text.len() == 0 {
                cx.answer("_No one reacted to this message\\._")
                    .reply_to_message_id(reply_to_message_id)
                    .parse_mode(ParseMode::MarkdownV2)
                    .send()
                    .await?;
            } else {
                cx.answer(text)
                    .reply_to_message_id(reply_to_message_id)
                    .send()
                    .await?;
            };
        }
    };

    Ok(())
}

async fn handle_query(cx: UpdateWithCx<CallbackQuery>) -> ResponseResult<()> {
    let query = cx.update;

    let msg = if let Some(msg) = query.message {
        msg
    } else {
        cx.bot
            .answer_callback_query(query.id)
            .text("Something's wrong with that message. Try this on another message.")
            .show_alert(true)
            .cache_time(1000)
            .send()
            .await?;
        return Ok(());
    };

    let text = if let Some(text) = msg.text() {
        text
    } else {
        cx.bot
            .answer_callback_query(query.id)
            .text("Something's wrong with that message. Try this on another message.")
            .show_alert(true)
            .cache_time(1000)
            .send()
            .await?;
        return Ok(());
    };

    let mut reactions_users = get_reactions_users(&msg).unwrap();

    let reaction = query.data.unwrap().parse().unwrap();
    let reaction_users = reactions_users
        .entry(reaction)
        .or_insert_with(|| HashSet::new());

    let mut reaction_removed = false;

    // toggle whether or not the user has this reaction
    if !reaction_users.insert(query.from.id) {
        reaction_users.remove(&query.from.id);
        reaction_removed = true;
    }

    let mut reaction_query_params = Vec::new();
    let mut reaction_display_params = Vec::new();

    for (reaction, users) in reactions_users {
        if users.len() > 0 {
            reaction_query_params.push(format!(
                "{}={}",
                reaction,
                users
                    .iter()
                    .map(|&id| lexical::to_string(id))
                    .collect::<Vec<_>>()
                    .join(",")
            ));

            reaction_display_params.push(format!("{} *{}*", reaction.get_emoji(), users.len()));
        }
    }

    let new_text = format!(
        "[\u{034f}](https://reaxnbot.dev/reactions?{}) {}",
        reaction_query_params.join("&"),
        reaction_display_params.join("  ")
    );

    cx.bot
        .edit_message_text(
            ChatOrInlineMessage::Chat {
                chat_id: ChatId::Id(msg.chat_id()),
                message_id: msg.id,
            },
            new_text,
        )
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(create_markup())
        .disable_web_page_preview(true)
        .send()
        .await?;

    cx.bot
        .answer_callback_query(query.id)
        .text(if reaction_removed {
            "❎"
        } else {
            reaction.get_emoji()
        })
        .send()
        .await?;

    Ok(())
}

fn create_markup() -> InlineKeyboardMarkup {
    let mut markup = InlineKeyboardMarkup::default();

    for reaction in &REACTIONS {
        markup = markup.append_to_row(
            InlineKeyboardButton::new(
                reaction.get_emoji(),
                InlineKeyboardButtonKind::CallbackData(reaction.to_string()),
            ),
            0,
        );
    }

    markup
}

fn get_reactions_users(msg: &Message) -> Option<HashMap<Reaction, HashSet<UserId>>> {
    msg.entities().map(|entities| {
        let mut reactions_users = HashMap::<Reaction, HashSet<i32>>::new();

        for entity in entities {
            match &entity.kind {
                MessageEntityKind::TextLink { url } => {
                    let url = Url::parse(url).unwrap();

                    if url.domain() != Some("reaxnbot.dev") || url.path() != "/reactions" {
                        continue;
                    }

                    for (reaction_id, user_ids) in url.query_pairs() {
                        let reaction = reaction_id.parse().unwrap();

                        reactions_users
                            .entry(reaction)
                            .or_insert_with(|| HashSet::new())
                            .extend(
                                user_ids
                                    .split(",")
                                    .map(lexical::parse)
                                    .collect::<Result<Vec<i32>, _>>()
                                    .unwrap(),
                            );
                    }
                }
                _ => {}
            }
        }

        reactions_users
    })
}
