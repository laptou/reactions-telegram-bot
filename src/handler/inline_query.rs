use std::collections::{HashMap, HashSet};

use teloxide::{prelude::*, types::*};

use crate::reaction::{Reaction, REACTIONS};

use super::util::{create_reaction_keyboard, get_reaction_message_text};

pub async fn handle_inline_query(cx: UpdateWithCx<InlineQuery>) -> anyhow::Result<()> {
    let user_id = cx.update.from.id;
    let users: HashSet<i32> = std::iter::once(user_id).collect();

    let stickers: Vec<InlineQueryResult> = REACTIONS
        .iter()
        .map(|reaction| {
            let reaction_users = (*reaction, users.clone());
            let text = get_reaction_message_text(std::iter::once(reaction_users));

            let sticker_id = match reaction {
                Reaction::Laugh => {
                    "CAACAgEAAx0CVdUh8wADBF_6BFDjvhQPJk_HcwTJef5gtdaGAAKyAwACHnNXBNRqRQfXrnlvHgQ"
                }
                Reaction::Anger => {
                    "CAACAgEAAx0CVdUh8wADBV_6BFmPXfkLjgKYDMsNnexNwyikAAKzAwACHnNXBNHDlDvdTaO_HgQ"
                }
                Reaction::Heart => {
                    "CAACAgEAAx0CVdUh8wADA1_6BDsyavcY6NBOb1SyCQAB4qAMSQACsQMAAh5zVwRIfhjCdqiVrh4E"
                }
                Reaction::Up => {
                    "CAACAgEAAx0CVdUh8wADB1_6BHoNtTO6WYQNR_uyIMSMaA-aAAK1AwACHnNXBHHZPfRvq2x6HgQ"
                }
                Reaction::Down => {
                    "CAACAgEAAx0CVdUh8wADB1_6BHoNtTO6WYQNR_uyIMSMaA-aAAK1AwACHnNXBHHZPfRvq2x6HgQ"
                }
                Reaction::Sad => {
                    "CAACAgEAAx0CVdUh8wADBl_6BGgLYALkCOfWR8cIYmC45_3sAAK0AwACHnNXBOzFJW9pN6veHgQ"
                }
            };

            let input_message_content = InputMessageContentText::new(text)
                .parse_mode(ParseMode::MarkdownV2)
                .disable_web_page_preview(true);

            let sticker = InlineQueryResultCachedSticker::new(reaction.to_string(), sticker_id)
                .input_message_content(InputMessageContent::Text(input_message_content))
                .reply_markup(create_reaction_keyboard());

            InlineQueryResult::CachedSticker(sticker)
        })
        .collect();

    cx.bot
        .answer_inline_query(cx.update.id, stickers)
        .cache_time(3000)
        .send()
        .await?;

    Ok(())
}
