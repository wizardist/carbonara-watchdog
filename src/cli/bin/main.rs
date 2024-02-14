const MENU_URL: &'static str =
    "https://www.raflaamo.fi/en/restaurant/helsinki/la-famiglia-helsinki/menu/lunch";

#[tokio::main]
async fn main() {
    let body = fetch_menu_json(MENU_URL);

    let json_data = body.await.unwrap();
    let v: serde_json::Value = serde_json::from_str(json_data.as_str()).unwrap();

    for week_menu in v["props"]["pageProps"]["initialApolloState"]["MenuListingItemRestaurant:369"]
        ["lunchMenuGroups"][0]["weeklyLunchMenu"]
        .as_array()
        .unwrap()
        .iter()
    {
        let start_date = &week_menu["date"]["start"];
        println!("{}", start_date);

        const WEEKDAYS: [&'static str; 5] =
            ["monday", "tuesday", "wednesday", "thursday", "friday"];

        for day in WEEKDAYS {
            let day_menu = &week_menu["dailyMenuAvailabilities"][day]["menu"];
            println!(
                "{}: {}",
                day, day_menu["menuSections"][0]["portions"][0]["name"]["default"]
            )
        }
    }
}

async fn fetch_menu_json(menu_url: &str) -> Result<String, reqwest::Error> {
    let body = reqwest::get(menu_url).await?.text().await?;

    const BEFORE: &str = r#"<script id="__NEXT_DATA__" type="application/json">"#;

    let start = body.find(BEFORE).unwrap() + BEFORE.len();
    let end = body[start..].find("</script>").unwrap();

    Ok(body[start..start + end].to_owned())
}
