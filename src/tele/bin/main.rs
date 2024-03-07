use carbonara_watchdog::carbo::*;
use chrono::{TimeDelta, Timelike, Utc};
use chrono_tz::Tz;
use core::time;
use std::collections::HashSet;
use std::error::Error;
use std::ops::Add;
use std::sync::{Arc, Once};
use teloxide::dptree::case;
use teloxide::types::Recipient;
use teloxide::{filter_command, prelude::*, utils::command::BotCommands};
use tokio::sync::RwLock;
use tokio::time::{sleep, sleep_until, Instant};

const TZ: &'static Tz = &Tz::Europe__Helsinki;
const ANNOUNCEMENT_HOUR: u32 = 9;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let bot = Bot::from_env();
    let subscribers: Arc<RwLock<HashSet<ChatId>>> = Arc::new(RwLock::new(HashSet::new()));

    tokio::spawn({
        let announce_bot = bot.clone();
        let announce_subscribers = subscribers.clone();

        async move {
            loop {
                sleep(time::Duration::new(300, 0)).await;
                sleep_until(get_next_announcement_instant()).await;

                let today = Utc::now().with_timezone(TZ).date_naive();
                let carbonara_date = get_next_carbonara_date(today).await.unwrap().unwrap();
                if today != carbonara_date {
                    continue;
                }

                let subs = announce_subscribers.read().await;

                for chat_id in subs.iter() {
                    announce_bot
                        .send_message(
                            Recipient::Id(chat_id.clone()),
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

fn get_next_announcement_instant() -> Instant {
    let now = Utc::now().with_timezone(TZ);
    let next = now
        .with_hour(ANNOUNCEMENT_HOUR)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();

    let start = Instant::now();

    let delta = next.signed_duration_since(now);
    if delta >= TimeDelta::zero() {
        start.add(delta.to_std().unwrap())
    } else {
        start.add(delta.add(TimeDelta::days(1)).to_std().unwrap())
    }
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
