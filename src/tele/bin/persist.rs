use std::process::exit;
use async_trait::async_trait;
use google_sheets4::{hyper, hyper_rustls, oauth2, Sheets};
use google_sheets4::api::{Scope, ValueRange};
use google_sheets4::oauth2::parse_application_secret;
use google_sheets4::oauth2::storage::{TokenInfo, TokenStorage};
use serde_json::{json};

pub async fn get_subscribers() -> Vec<i64> {
    let b64_token = std::env::var("SHEETS_APP_SECRET").unwrap_or(String::new());
    let b64_vec = base64::decode(b64_token.as_bytes()).unwrap();
    let app_secret = String::from_utf8(b64_vec).unwrap();

    let secret: oauth2::ApplicationSecret = parse_application_secret(app_secret).unwrap();
    let store = Box::new(EnvSecretStorage{});
    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::Interactive,
    ).with_storage(store).build().await.unwrap();
    let hub = Sheets::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_only().enable_http1().build()), auth);

    let spreadsheet_id = std::env::var("SHEETS_DOCUMENT_ID").unwrap();

    let result = hub.spreadsheets()
        .values_get(spreadsheet_id.as_str(), "subscriptions")
        .doit().await;

    let x = result.unwrap().1.values.unwrap();

    // Reads the first column of each row as u64 to a vector of subscription IDs
    let subscription_ids: Vec<_> = x.iter()
        .filter_map(|row| {
            match row.first() {
                None => None,
                Some(v) => {
                    let str_id = v.as_str().unwrap_or("").trim();

                    if str_id.is_empty() {
                        return None;
                    }

                    String::from(str_id).parse::<i64>().ok()
                }
            }
        })
        // TODO: unique
        .collect();

    subscription_ids
}

pub async fn store_subscriber(subscriber_id: i64) {
    let b64_token = std::env::var("SHEETS_APP_SECRET").unwrap_or(String::new());
    let b64_vec = base64::decode(b64_token.as_bytes()).unwrap();
    let app_secret = String::from_utf8(b64_vec).unwrap();

    let secret: oauth2::ApplicationSecret = parse_application_secret(app_secret).unwrap();
    let store = Box::new(EnvSecretStorage{});
    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::Interactive,
    ).with_storage(store).build().await.unwrap();
    let hub = Sheets::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_only().enable_http1().build()), auth);

    let spreadsheet_id = std::env::var("SHEETS_DOCUMENT_ID").unwrap();

    // subscriptions!A1 is used to anchor Google Sheets API to work with 
    // the topmost-left table. Specifying just "subscriptions" would leave
    // the API to detect the last table as described in
    // https://developers.google.com/sheets/api/guides/values#append_values
    let range = ValueRange {
        major_dimension: Some(String::from("ROWS")),
        range: Some(String::from("subscriptions!A1")),
        values: Some(vec![vec![json!(subscriber_id)]]),
    };
    let r = hub.spreadsheets()
        .values_append(range, spreadsheet_id.as_str(), "subscriptions!A1")
        .value_input_option("RAW")
        .insert_data_option("INSERT_ROWS")
        .add_scope(Scope::Drive)
        .doit().await;

    if let Err(e) = r {
        println!("{:?}", e);
        exit(1);
    }
}

struct EnvSecretStorage {}

#[async_trait]
impl TokenStorage for EnvSecretStorage {
    async fn set(&self, _scopes: &[&str], _token: TokenInfo) -> anyhow::Result<()> {
        // println!("{:?}", base64::encode(serde_json::to_string(&_token).unwrap().as_bytes()));
        Ok(())
    }

    async fn get(&self, _scopes: &[&str]) -> Option<TokenInfo> {
        // Ignore scopes as we only need one set of stored credentials.
        // Just get them from the environment.
        let b64_token = std::env::var("SHEETS_TOKEN").unwrap_or(String::new());
        let b64_vec = base64::decode(b64_token.as_bytes()).unwrap();
        let json_token = String::from_utf8(b64_vec).unwrap();
        if let Ok(ti) = serde_json::from_str(json_token.as_str()) {
            return Some(ti)
        }

        None
    }
}
