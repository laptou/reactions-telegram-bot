/* eslint-disable @typescript-eslint/camelcase */
import Telegraf from 'telegraf';
import type * as tt from 'telegram-typings';
import Pino from 'pino';

const dev = process.env.NODE_ENV === 'development';
const token = process.env.BOT_TOKEN;
const logger = Pino();

if (!token) {
  logger.error('Must supply BOT_TOKEN environment variable with bot token inside.');
  process.exit(1);
}

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

bot.command('r', (ctx, next) => {
  if (ctx.message?.reply_to_message) {
    bot.telegram.sendMessage(ctx.chat!.id, '\u034f', {
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
});

bot.use(async (ctx, next) => {
  if (ctx.callbackQuery) {
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

      if (reaction !== newReaction && idIndex !== -1) {
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

    await ctx.answerCbQuery();
  }

  next?.();
});

bot.launch();