# Reaction Bot

Facebook Messenger and iMessage have features that allow you to react to
messages with emojis - it's an inline, low-key way to acknowledge a message when
you don't have anything else to say. However, this feature is still missing from
Telegram, so this bot serves as a sort of polyfill.

## How to use it (for Telegram users)

- Add [@reaxnbot](https://t.me/reaxnbot) to a group chat
- Reply to any message with the `/r` command directed at `@reaxnbot` (you can
  just type `/r` and it should just work)
- `@reaxnbot` responds with an empty message with emoji buttons, and anyone can
  tap them to react

![example showing reaxnbot in action](doc/example-1.png)

- If you don't want your chat littered with `/r`s, then give the bot permissions
  to delete messages and it will automatically delete the `/r` messages.
- If you want to see who did which reaction, reply to a reaction message with
  the `/s` command.

## How to use it (for developers)

- Obtain a bot token from the [BotFather](https://t.me/botfather)
- `TELOXIDE_TOKEN=<your token here> cargo run`

## EAQ (Easily anticipated questions)

> Why are you storing the reactions using that weird Markdown hack instead of
> just using a database like a normal person?

I do not want to maintain a database. This bot needs to be something that will
cost me nothing to operate, whether that's time or money, and databases always
cost money eventually.

Also, this puts the reaction data in the hands of Telegram, whom you probably
already trust, instead of the hands of me.

> Why does this bot have the ability to read all messages in my group chat?

Due to the [way the Telegram Bot API works](https://core.telegram.org/bots/faq#what-messages-will-my-bot-get),
replying to a message with a bot command doesn't let the bot see which message
you're replying to. This bot would be a lot less useful if it wasn't clear which
message the reaction was for, so this is a no-go.

The bot completely ignores messages that are not commands directed at it, and
does not read the contents of the messages you use it to react to.

> I don't want this bot to delete my `/r` messages. / I don't trust this bot not
> to delete anything else.

Then don't give the bot deletion permissions.

> Didn't this project used to be written in TypeScript?

Yes, but I found that frustrating because essentially any operation that one can
do with the Telegram Bot API can fail, and with so many potential places for
errors to creep up, I preferred Rust's error handling strategy to TypeScript's.

Additionally, the objects that the Telegram API returns are highly polymorphic,
and dealing with them in TypeScript did not feel explicit enough for my tastes.

Moreover, the program will (probably) use fewer resources as written in Rust,
which is a win for me if it suddenly finds a large audience.

And finally, I just wanted some more practice writing programs in Rust.

## License

This code is MIT licensed. Copyright &copy; 2020 Ibiyemi Abiodun.
