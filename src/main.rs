use std::{
    collections::HashMap, collections::HashSet, convert::Infallible, env, fmt::Display,
    str::FromStr, sync::Arc,
};

use anyhow::Context;
use futures::future::join_all;
use lexical;
use log::{info, warn};
use pretty_env_logger;
use teloxide::utils::command::BotCommand;
use teloxide::{prelude::*, types::*};
use tokio::{self};
use url::Url;

#[derive(BotCommand, PartialEq, Eq, Debug, Clone, Copy)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "react to a message", rename = "r")]
    React,
    #[command(description = "react to a message with a ❤")]
    Heart,
    #[command(description = "react to a message with a 👍")]
    Up,
    #[command(description = "react to a message with a 👎")]
    Down,
    #[command(description = "show who reacted to a message", rename = "s")]
    Show,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Reaction {
    Laugh,
    Anger,
    Heart,
    Up,
    Down,
    Sad,
}

static REACTIONS: [Reaction; 6] = [
    Reaction::Heart,
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
            "love" => Reaction::Heart,
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
            Reaction::Heart => write!(f, "love"),
            Reaction::Up => write!(f, "up"),
            Reaction::Down => write!(f, "down"),
            Reaction::Sad => write!(f, "sad"),
        }
    }
}

impl Reaction {
    pub fn get_emoji(&self) -> &str {
        match self {
            Reaction::Heart => "❤️",
            Reaction::Laugh => "😂",
            Reaction::Anger => "😡",
            Reaction::Sad => "😭",
            Reaction::Up => "👍",
            Reaction::Down => "👎",
        }
    }
}

type UserId = i32;

#[tokio::main]
async fn main() {
    info!("starting reactions bot");

    teloxide::enable_logging!();

    info!("token: {}", env::var("TELOXIDE_TOKEN").unwrap());

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
                if let Err(err) = handle_callback_query(update).await {
                    warn!("callback query response failed: {:?}", err);
                }
            })
        })
        // .inline_queries_handler(move |rx: DispatcherHandlerRx<InlineQuery>| {
        //     rx.for_each_concurrent(None, move |update| async move {
        //         if let Some(text) = update.update.query {
        //             if let Ok(cmd) = Command::parse(text, "reaxnbot") {
        //                 if let Err(err) = handle_inline_query(update, cmd).await {
        //                     warn!("message response failed: {:?}", err);
        //                 }
        //             }
        //         }
        //     })
        // })
        .dispatch()
        .await;
}

async fn handle_command(cx: UpdateWithCx<Message>, command: Command) -> anyhow::Result<()> {
    match command {
        Command::Help => {
            cx.answer(Command::descriptions()).send().await?;
        }
        Command::React | Command::Up | Command::Down | Command::Heart => {
            let reply_to_message = {
                match cx.update.reply_to_message() {
                    Some(message) => message,
                    None => {
                        cx.reply_to("You need to reply to a message with /r so I know which message you're reacting to.").send().await?;
                        return Ok(());
                    }
                }
            };

            match cx.delete_message().send().await {
                Ok(_) => {}
                Err(RequestError::ApiError { status_code, .. }) if status_code.as_u16() == 401 => {
                    // continue silently if we don't have permissions to
                    // delete /r messages 401 status codes probably mean the
                    // bot doesn't have permissions
                }
                Err(err) => return Err(err.into()),
            };

            let response = cx
                .answer("\u{034f}")
                .reply_to_message_id(reply_to_message.id)
                .reply_markup(create_reaction_keyboard())
                .disable_notification(true)
                .disable_web_page_preview(true)
                .send()
                .await?;

            // if this was a shortcut command to a specific reaction, put
            // that reaction in
            if command != Command::React {
                let user = cx
                    .update
                    .from()
                    .context("could not get user that issued the command")?;

                let reaction = match command {
                    Command::Heart => Reaction::Heart,
                    Command::Up => Reaction::Up,
                    Command::Down => Reaction::Down,
                    _ => unreachable!(),
                };

                toggle_reaction(&cx.bot, reaction, &response, user.id).await?;
            }
        }
        Command::Show => {
            let reply_to_message = {
                match cx.update.reply_to_message() {
                    Some(message) => message,
                    None => {
                        cx.reply_to("You need to reply to a message with /s so I know which message you're asking about.").send().await?;
                        return Ok(());
                    }
                }
            };

            let is_reaction_message = reply_to_message.from().map_or(false, |user| {
                user.is_bot && (user.username).as_deref().unwrap() == "reaxnbot"
            }) && reply_to_message
                .text()
                .map_or(false, |text| text.contains("\u{034f}"));

            if !is_reaction_message {
                cx.reply_to("You can only use /s to reply to a reaction message.")
                    .send()
                    .await?;
                return Ok(());
            }

            let chat_id = reply_to_message.chat_id();
            let reply_to_message_id = reply_to_message.id;
            let mut reactions: Vec<(Reaction, Vec<String>)> = vec![];

            // this loop was originally done in parallel with a bunch of joins
            // and async move closures but this is easier to read
            
            for (reaction, user_ids) in get_reactions_users(&reply_to_message).unwrap() {
                let mut user_names = vec![];

                for user_id in user_ids {
                    let user_name = match cx.bot.get_chat_member(chat_id, user_id).send().await {
                        Ok(user) => user.user.full_name(),
                        Err(_) => "(unknown)".to_owned(),
                    };

                    user_names.push(user_name);
                }

                reactions.push((reaction, user_names));
            }

            // respond to the user
            if reactions.len() == 0 {
                cx.answer("_No one reacted to this message\\._")
                    .reply_to_message_id(reply_to_message_id)
                    .parse_mode(ParseMode::MarkdownV2)
                    .send()
                    .await?;
            } else {
                let text = reactions
                    .into_iter()
                    .map(|(reaction, user_names)| {
                        format!("{} — {}", reaction.get_emoji(), user_names.join(", "))
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                cx.answer(text)
                    .reply_to_message_id(reply_to_message_id)
                    .send()
                    .await?;
            };
        }
    };

    Ok(())
}

async fn handle_callback_query(cx: UpdateWithCx<CallbackQuery>) -> anyhow::Result<()> {
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

    let reaction = query.data.unwrap().parse().unwrap();

    let reaction_added = toggle_reaction(&cx.bot, reaction, &msg, query.from.id).await?;

    cx.bot
        .answer_callback_query(query.id)
        .text(if reaction_added {
            reaction.get_emoji()
        } else {
            "❎"
        })
        .send()
        .await?;

    Ok(())
}

async fn handle_inline_query(cx: UpdateWithCx<InlineQuery>) -> ResponseResult<()> {
    todo!()
}

/// Toggles the user's reaction to this message. Returns true if the reaction
/// was added, and false if it was removed.
async fn toggle_reaction(
    bot: &Bot,
    reaction: Reaction,
    message: &Message,
    user_id: i32,
) -> anyhow::Result<bool> {
    let mut reactions_users = get_reactions_users(&message).unwrap();

    let reaction_users = reactions_users
        .entry(reaction)
        .or_insert_with(|| HashSet::new());

    let mut reaction_added = true;

    // toggle whether or not the user has this reaction
    if !reaction_users.insert(user_id) {
        reaction_users.remove(&user_id);
        reaction_added = false;
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

    bot.edit_message_text(
        ChatOrInlineMessage::Chat {
            chat_id: ChatId::Id(message.chat_id()),
            message_id: message.id,
        },
        new_text,
    )
    .parse_mode(ParseMode::MarkdownV2)
    .reply_markup(create_reaction_keyboard())
    .disable_web_page_preview(true)
    .send()
    .await?;

    Ok(reaction_added)
}

fn create_reaction_keyboard() -> InlineKeyboardMarkup {
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
