use carbonara_watchdog::carbo::*;
use chrono::{TimeDelta, Timelike, Utc};
use chrono_tz::Tz;
use std::collections::HashSet;
use std::ops::Add;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use teloxide::dptree::case;
use teloxide::{filter_command, prelude::*, utils::command::BotCommands};
use tokio::time::{interval_at, Instant};

const TZ: &'static Tz = &Tz::Europe__Helsinki;
const ANNOUNCEMENT_HOUR: u32 = 9;
const ANNOUNCEMENT_RATE: u64 = 24 * 60 * 60;

#[tokio::main]
async fn main() {
    let today = Utc::now().with_timezone(TZ).date_naive();
    let v = get_next_carbonara_date(today).await;

    let carbonara_date = v.unwrap().unwrap();

    println!("{}", carbonara_date);

    let subscribers: Arc<RwLock<HashSet<ChatId>>> = Arc::new(RwLock::new(HashSet::new()));

    tokio::spawn(async move {
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

            // TODO: Send messages to chats
        }
    });

    tokio::spawn(async {
        let bot = Bot::from_env();

        let cmd_handler =
            filter_command::<Command, _>().branch(case![Command::Subscribe].endpoint(subscribe));

        let msg_handler = Update::filter_message().branch(cmd_handler);

        Dispatcher::builder(bot, msg_handler)
            .dependencies(dptree::deps![subscribers])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    })
    .await
    .unwrap();
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

async fn subscribe(
    bot: Bot,
    subscribers: Arc<RwLock<HashSet<ChatId>>>,
    msg: Message,
) -> ResponseResult<()> {
    subscribers.write().unwrap().insert(msg.chat.id);
    bot.send_message(
        msg.chat.id,
        "I will tell you about carbonara lunch in the morning.",
    )
    .await?;

    Ok(())
}
