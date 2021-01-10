use std::{convert::Infallible, env, net::SocketAddr};

use log::{info, warn};
use pretty_env_logger;
use teloxide::{dispatching::update_listeners::UpdateListener, utils::command::BotCommand};
use teloxide::{prelude::*, types::*};
use tokio;

mod handler;
mod reaction;

use handler::{handle_callback_query, handle_command};
use warp::{hyper::StatusCode, Filter};

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

    info!("token: {:?}", env::var("TELOXIDE_TOKEN").unwrap());

    let bot = Bot::from_env();

    let listener = webhook(bot.clone()).await;

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
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}

pub async fn webhook<'a>(bot: Bot) -> impl UpdateListener<Infallible> {
    // Heroku defines auto defines a port value
    let teloxide_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN env variable missing");
    let port: u16 = env::var("PORT")
        .expect("PORT env variable missing")
        .parse()
        .expect("PORT value to be integer");

    let endpoint = format!("bot{}", teloxide_token);
    let url = format!("https://reaxnbot.dev/reaction/{}", endpoint);

    bot.set_webhook(url)
        .send()
        .await
        .expect("Cannot setup a webhook");

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::path("reaction"))
        .and(warp::path(endpoint))
        .and(warp::body::json())
        .map(move |json: serde_json::Value| {
            let try_parse = match serde_json::from_str(&json.to_string()) {
                Ok(update) => Ok(update),
                Err(error) => {
                    log::error!(
                        "Cannot parse an update.\nError: {:?}\nValue: {}\n\
                       This is a bug in teloxide, please open an issue here: \
                       https://github.com/teloxide/teloxide/issues.",
                        error,
                        json
                    );
                    Err(error)
                }
            };
            if let Ok(update) = try_parse {
                tx.send(Ok(update))
                    .expect("Cannot send an incoming update from the webhook")
            }

            StatusCode::OK
        });

    let test_route = warp::get()
        .and(warp::path::tail())
        .map(|tail| format!("hello world {:?}", tail));

    let server = server.or(test_route).recover(handle_rejection);

    let serve = warp::serve(server);

    let address = format!("0.0.0.0:{}", port);
    tokio::spawn(serve.run(address.parse::<SocketAddr>().unwrap()));
    rx
}

async fn handle_rejection(error: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    log::error!("Cannot process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}
