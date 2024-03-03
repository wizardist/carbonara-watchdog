use chrono::*;
use std::ops::Add;

const MENU_URL: &'static str =
    "https://www.raflaamo.fi/en/restaurant/helsinki/la-famiglia-helsinki/menu/lunch";

/// Allows to calculate Friday for weeks starting on Monday.
const FRIDAY_OFFSET: u64 = 4;

pub async fn get_next_carbonara_date(today: NaiveDate) -> Result<Option<NaiveDate>, String> {
    let body = fetch_menu_json(MENU_URL);

    let json_data = body.await.unwrap();
    let v: serde_json::Value = serde_json::from_str(json_data.as_str()).unwrap();

    let friday = |d: NaiveDate| {
        d.week(Weekday::Mon)
            .first_day()
            .add(Days::new(FRIDAY_OFFSET))
    };

    for week_menu in v["props"]["pageProps"]["initialApolloState"]["MenuListingItemRestaurant:369"]
        ["lunchMenuGroups"][0]["weeklyLunchMenu"]
        .as_array()
        .unwrap()
        .iter()
    {
        let start_date =
            NaiveDate::parse_from_str(&week_menu["date"]["start"].as_str().unwrap(), "%Y-%m-%d")
                .unwrap();

        // We don't need to check the week if today is past week's Friday or the whole week has passed.
        let week_friday = friday(start_date);
        if today.signed_duration_since(week_friday) > TimeDelta::zero() {
            continue;
        }

        const WEEKDAYS: [&'static str; 5] =
            ["monday", "tuesday", "wednesday", "thursday", "friday"];

        for (day_offset, day) in WEEKDAYS.iter().enumerate() {
            let dish_date = start_date.add(Days::new(day_offset as u64));

            // We don't need to check days that precede today.
            if today.signed_duration_since(dish_date) > TimeDelta::zero() {
                continue;
            }

            let day_menu = &week_menu["dailyMenuAvailabilities"][day]["menu"];
            let dish = day_menu["menuSections"][0]["portions"][0]["name"]["default"]
                .as_str()
                .to_owned()
                .unwrap();

            if dish.to_lowercase().contains("carbonara") {
                return Ok(Some(dish_date));
            }
        }
    }

    Ok(None)
}

async fn fetch_menu_json(menu_url: &str) -> Result<String, reqwest::Error> {
    let body = reqwest::get(menu_url).await?.text().await?;

    const BEFORE: &str = r#"<script id="__NEXT_DATA__" type="application/json">"#;

    let start = body.find(BEFORE).unwrap() + BEFORE.len();
    let end = body[start..].find("</script>").unwrap();

    Ok(body[start..start + end].to_owned())
}
