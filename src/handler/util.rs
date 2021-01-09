use std::collections::{HashMap, HashSet};
use teloxide::{prelude::*, types::*};
use url::Url;

use crate::reaction::{Reaction, REACTIONS};

/// Creates inline keyboard markup for the reactions available to the user.
pub fn create_reaction_keyboard() -> InlineKeyboardMarkup {
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

/// Gets the reactions encoded in a message, and returns a mapping from
/// reactions to user IDs. Returns None if this message doesn't encode any
/// reactions.
pub fn get_reactions_users(msg: &Message) -> Option<HashMap<Reaction, HashSet<i32>>> {
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

/// Toggles the user's reaction to this message. Returns true if the reaction
/// was added, and false if it was removed.
pub async fn toggle_reaction(
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
