use std::{collections::HashSet, env};

use log::{info, warn};
use pretty_env_logger;
use teloxide::utils::command::BotCommand;
use teloxide::{prelude::*, types::*};
use tokio;

mod handler;
mod reaction;

use handler::{handle_callback_query, handle_command};

#[derive(BotCommand, PartialEq, Eq, Debug, Clone, Copy)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
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
