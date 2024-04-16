mod persist;

use carbonara_watchdog::carbo::*;
use chrono::Utc;
use chrono_tz::Tz;

const TZ: &'static Tz = &Tz::Europe__Helsinki;

#[tokio::main]
async fn main() {
    let today = Utc::now().with_timezone(TZ).date_naive();
    let v = get_next_carbonara_date(today).await;

    let carbonara_date = v.unwrap().unwrap();

    println!("{}", carbonara_date);

    persist::get_subscribers().await;
}
