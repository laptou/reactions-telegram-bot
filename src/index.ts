/* eslint-disable @typescript-eslint/camelcase */
import type * as tt from 'telegram-typings';
import Pino from 'pino';
import Telegraf from 'telegraf';

const token = process.env.BOT_TOKEN;
const logger = Pino();

if (!token) {
  logger.error('Must supply BOT_TOKEN environment variable with bot token inside.');
  process.exit(1);
}

logger.info('starting Reactions bot');

enum Reaction {
  Heart = 'heart',
  Approve = 'approve',
  Disapprove = 'disapprove',
  Laugh = 'laugh',
  Anger = 'anger',
  Sad = 'sad'
}

interface ButtonData {
  reaction: Reaction;
  users: number[];
}

const REACTION_VALUES = Object.values(Reaction);
const REACTION_EMOJIS = {
  [Reaction.Heart]: '❤',
  [Reaction.Approve]: '👍',
  [Reaction.Disapprove]: '👎',
  [Reaction.Laugh]: '😂',
  [Reaction.Anger]: '😠',
  [Reaction.Sad]: '😢'
};

const bot = new Telegraf(token);

function tryParseJSON(str: string | null | undefined): any {
  if (str === null || str === undefined) return null;

  try {
    return JSON.parse(str);
  } catch {
    return null;
  }
}

(async () => {
  const self = await bot.telegram.getMe();

  bot.command('r', (ctx, next) => {
    try {
      if (
        ctx.message?.reply_to_message &&
        ctx.message?.reply_to_message.from?.id === self.id) {
        logger.trace('received /r', { message: ctx.message, chat: ctx.chat });
        bot.telegram.sendMessage(ctx.chat!.id, '\u034f', {
          disable_notification: true,
          reply_to_message_id: ctx.message.reply_to_message.message_id,
          reply_markup: {
            inline_keyboard: [
              REACTION_VALUES.map(v => ({
                text: REACTION_EMOJIS[v],
                callback_data: JSON.stringify({ reaction: v, users: [] })
              }))
            ]
          }
        });
      }

      ctx.deleteMessage();

      next?.();
    } catch (error) {
      logger.error('error on processing /r', { error });
    }
  });

  // respond to button click
  bot.use(async (ctx, next) => {
    if (ctx.callbackQuery) {
      try {
        const { callbackQuery } = ctx;

        const callbackData = tryParseJSON(callbackQuery.data);
        if (!callbackData) {
          logger.warn('callbackData was null', { ctx });
          return;
        }

        const { reaction: newReaction } = callbackData;
        if (!REACTION_VALUES.includes(newReaction)) {
          logger.warn('received invalid reaction', {
            reaction: newReaction,
            ctx
          });

          return;
        }

        const message = callbackQuery.message;
        if (!message) return;

        const { inline_keyboard: inlineKeyboard } = (callbackQuery.message as any).reply_markup as tt.InlineKeyboardMarkup;

        // parse the old inline keyboard and make a new one
        const reactionUsers = Object.fromEntries(inlineKeyboard[0].map(button => {
          const buttonCallbackData = tryParseJSON(button.callback_data) as ButtonData;
          const { reaction, users } = buttonCallbackData;

          const idIndex = users.indexOf(callbackQuery.from.id);
          if (reaction === newReaction && idIndex === -1) {
            users.push(callbackQuery.from.id);
          }

          if (idIndex !== -1) {
            users.splice(idIndex, 1);
          }

          return [reaction, users] as const;
        }));

        await ctx.editMessageReplyMarkup({
          inline_keyboard: [
            REACTION_VALUES.map(v => {
              return {
                text: reactionUsers[v].length > 0 ?
                  REACTION_EMOJIS[v] + ' ' + reactionUsers[v].length :
                  REACTION_EMOJIS[v],
                callback_data: JSON.stringify({ reaction: v, users: reactionUsers[v] })
              };
            })
          ]
        });
      } catch (error) {
        logger.error('error on processing callback query', { error });
      }

      try {
        // this can fail if we took too long to respond
        await ctx.answerCbQuery();
      } catch {
        logger.warn('failed to answer cb query', { callbackQuery });
      }
    }

    next?.();
  });

  await bot.launch();

  logger.info('launched Reactions bot');
})();