use anyhow::Context;
use teloxide::utils::command::BotCommand;
use teloxide::{prelude::*, types::*};

use crate::{reaction::Reaction, Command};

use super::util::{create_reaction_keyboard, get_reactions_users, toggle_reaction};

pub async fn handle_command(cx: UpdateWithCx<Message>, command: Command) -> anyhow::Result<()> {
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
