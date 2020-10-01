use async_compat::CompatExt;
use log::info;
use pretty_env_logger;
use rmp_serde;
use serde::{Deserialize, Serialize};
use smol;
use teloxide::utils::command::BotCommand;
use teloxide::{prelude::*, types::*};

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "react to a message", rename = "r")]
    React,
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
enum Reaction {
    Laugh,
    Anger,
    Love,
    Up,
    Down,
    Sad,
    Custom(char),
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

fn main() {
    teloxide::enable_logging!();
    info!("Starting Reactions bot");

    let bot = Bot::from_env();
    smol::block_on(async {
        teloxide::commands_repl(bot, "Reactions Bot", answer)
            .compat()
            .await;
    })
}

async fn answer(cx: UpdateWithCx<Message>, command: Command) -> ResponseResult<()> {
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
