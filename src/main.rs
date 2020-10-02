use std::{borrow::Cow, collections::HashMap, collections::HashSet};

use lexical;
use log::{debug, error, info, warn};
use pretty_env_logger;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash)]
enum Reaction {
    Laugh,
    Anger,
    Love,
    Up,
    Down,
    Sad,
    Custom(char),
}

impl<'a, S: Into<&'a str>> From<S> for Reaction {
    fn from(string: S) -> Self {
        match string.into() {
            "laugh" => Reaction::Laugh,
            "anger" => Reaction::Anger,
            "love" => Reaction::Love,
            "up" => Reaction::Up,
            "down" => Reaction::Down,
            "sad" => Reaction::Sad,
            _ => unimplemented!(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ReactionData {
    reaction: Reaction,
    ids: Vec<i32>,
}

impl ReactionData {
    fn new(reaction: Reaction) -> Self {
        ReactionData {
            reaction,
            ids: vec![],
        }
    }
}

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
            debug!("/r received");
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
                .reply_markup({
                    let mut markup = InlineKeyboardMarkup::default();
                    let data = ReactionData::new(Reaction::Love);

                    markup.append_to_row(
                        InlineKeyboardButton::new(
                            "❤",
                            InlineKeyboardButtonKind::CallbackData("e".to_owned()),
                        ),
                        0,
                    )
                })
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

    let mut pairs = HashMap::<Reaction, HashSet<i32>>::new();

    if let Some(entities) = msg.entities() {
        for entity in entities {
            match &entity.kind {
                MessageEntityKind::TextLink { url } => {
                    let url = Url::parse(url).unwrap();

                    if url.domain() != Some("reaxnbot.dev") || url.path() != "/reactions" {
                        continue;
                    }

                    url.query_pairs()
                        .map(move |(user_id, reaction_id)| {
                            (
                                reaction_id.as_ref().into(),
                                lexical::parse(user_id.as_bytes()).unwrap(),
                            )
                        })
                        .fold(&mut pairs, |pairs, (k, v)| {
                            pairs.entry(k).or_insert_with(|| HashSet::new()).insert(v);
                            pairs
                        });
                }
                _ => {}
            }
        }
    }

    

    let new_text = format!(
        "[\u{034f}](https://reaxnbot.dev/reactions?{})",
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
        .reply_markup(
            msg.reply_markup()
                .expect("got a callback query on a message with no reply markup")
                .clone(),
        )
        .send()
        .await?;

    cx.bot.answer_callback_query(query.id).send().await?;

    Ok(())
}
