use std::{
    borrow::Cow, collections::HashMap, collections::HashSet, convert::Infallible, fmt::Display,
    str::FromStr,
};

use lexical;
use log::{debug, error, info, warn};
use pretty_env_logger;
use teloxide::utils::command::BotCommand;
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
                    if let Ok(cmd) = Command::parse(text, "Reactions Bot") {
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

            cx.delete_message().send().await?;

            cx.answer("\u{034f}")
                .reply_to_message_id(reply_to_message.id)
                .reply_markup(create_markup(HashMap::new()))
                .send()
                .await?;
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

    let mut reactions_users = HashMap::<Reaction, HashSet<i32>>::new();

    if let Some(entities) = msg.entities() {
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
    }

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

    let new_text = format!(
        "[\u{034f}](https://reaxnbot.dev/reactions?{})",
        reactions_users
            .iter()
            .filter_map(|(reaction, users)| if users.len() > 0 {
                Some(format!(
                    "{}={}",
                    reaction,
                    users
                        .iter()
                        .map(|&id| lexical::to_string(id))
                        .collect::<Vec<_>>()
                        .join(",")
                ))
            } else {
                None
            })
            .collect::<Vec<_>>()
            .join("&")
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
        .reply_markup(create_markup(reactions_users))
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

fn create_markup(reactions_users: HashMap<Reaction, HashSet<UserId>>) -> InlineKeyboardMarkup {
    let mut markup = InlineKeyboardMarkup::default();

    for reaction in &REACTIONS {
        let btn_text =
            reactions_users
                .get(&reaction)
                .map_or(reaction.get_emoji().to_owned(), |users| {
                    if users.len() > 0 {
                        format!("{} {}", reaction.get_emoji(), users.len())
                    } else {
                        reaction.get_emoji().to_owned()
                    }
                });

        markup = markup.append_to_row(
            InlineKeyboardButton::new(
                btn_text,
                InlineKeyboardButtonKind::CallbackData(reaction.to_string()),
            ),
            0,
        );
    }

    markup
}
