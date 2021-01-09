use teloxide::{prelude::*, types::CallbackQuery};

use super::util::toggle_reaction;

pub async fn handle_callback_query(cx: UpdateWithCx<CallbackQuery>) -> anyhow::Result<()> {
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
