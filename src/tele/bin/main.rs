use carbonara_watchdog::carbo::*;
use chrono::{TimeDelta, Timelike, Utc};
use chrono_tz::Tz;
use std::collections::HashSet;
use std::error::Error;
use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;
use teloxide::dptree::case;
use teloxide::types::Recipient;
use teloxide::{filter_command, prelude::*, utils::command::BotCommands};
use tokio::sync::RwLock;
use tokio::time::{interval_at, Instant};

const TZ: &'static Tz = &Tz::Europe__Helsinki;
const ANNOUNCEMENT_HOUR: u32 = 9;
const ANNOUNCEMENT_RATE: u64 = 24 * 60 * 60;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let today = Utc::now().with_timezone(TZ).date_naive();
    let v = get_next_carbonara_date(today).await;

    let carbonara_date = v.unwrap().unwrap();

    println!("{}", carbonara_date);

    let subscribers: Arc<RwLock<HashSet<ChatId>>> = Arc::new(RwLock::new(HashSet::new()));

    let bot = Bot::from_env();

    tokio::spawn({
        let announce_bot = bot.clone();
        let announce_subscribers = subscribers.clone();

        async move {
            let n = Utc::now().with_timezone(TZ);
            let ps = n
                .with_hour(ANNOUNCEMENT_HOUR)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap();

            let mut start = Instant::now();

            let delta = ps.signed_duration_since(n);
            if delta >= TimeDelta::zero() {
                start = start.add(delta.to_std().unwrap());
            } else {
                let delta = delta.add(TimeDelta::days(1));
                start = start.add(delta.to_std().unwrap());
            }

            let mut announcement_ticker = interval_at(start, Duration::new(ANNOUNCEMENT_RATE, 0));

            loop {
                announcement_ticker.tick().await;

                let subs = announce_subscribers.read().await;

                for x in subs.iter() {
                    announce_bot
                        .send_message(
                            Recipient::Id(x.clone()),
                            r#"
🇮🇹🤌🍝 Today is the day! 🍝🤌🇮🇹

No guessing your lunch choice today. Carbonara is on the menu at La Famiglia.

TORILLE! 🇫🇮
"#,
                        )
                        .await
                        .unwrap();
                }
            }
        }
    });

    tokio::spawn({
        let command_bot = bot.clone();
        let command_subscribers = subscribers.clone();

        async {
            let cmd_handler = filter_command::<Command, _>()
                .branch(case![Command::Subscribe].endpoint(subscribe));

            let msg_handler = Update::filter_message().branch(cmd_handler);

            Dispatcher::builder(command_bot, msg_handler)
                .dependencies(dptree::deps![command_subscribers])
                .enable_ctrlc_handler()
                .build()
                .dispatch()
                .await;
        }
    })
    .await
    .unwrap();

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "I support the following commands:"
)]
enum Command {
    #[command(description = "Subscribe to carbonara announcements")]
    Subscribe,
}

/// Handles Command::Subscribe.
async fn subscribe(
    bot: Bot,
    subscribers: Arc<RwLock<HashSet<ChatId>>>,
    msg: Message,
) -> ResponseResult<()> {
    subscribers.write().await.insert(msg.chat.id);
    bot.send_message(
        msg.chat.id,
        "I will tell you about carbonara lunch in the morning.",
    )
    .await?;

    Ok(())
}
