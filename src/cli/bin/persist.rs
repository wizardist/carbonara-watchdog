use async_trait::async_trait;
use google_sheets4::{hyper, hyper_rustls, oauth2, Sheets};
use google_sheets4::oauth2::parse_application_secret;
use google_sheets4::oauth2::storage::{TokenInfo, TokenStorage};

pub async fn get_subscribers() {
    let secret: oauth2::ApplicationSecret = parse_application_secret(r#""#).unwrap();
    let store = Box::new(EnvSecretStorage{});
    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::Interactive,
    ).with_storage(store).build().await.unwrap();
    let hub = Sheets::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_only().enable_http1().build()), auth);

    let result = hub.spreadsheets()
        .values_get("", "A1:A5")
        .doit().await;

    println!("{:?}", result.unwrap().1.values.unwrap());
}

struct EnvSecretStorage {}

#[async_trait]
impl TokenStorage for EnvSecretStorage {
    async fn set(&self, scopes: &[&str], token: TokenInfo) -> anyhow::Result<()> {
        println!("{:?}", base64::encode(serde_json::to_string(&token).unwrap().as_bytes()));
        Ok(())
    }

    async fn get(&self, scopes: &[&str]) -> Option<TokenInfo> {
        let b64_token = std::env::var("SHEETS_TOKEN").unwrap_or(String::new());
        let b64_vec = base64::decode(b64_token.as_bytes()).unwrap();
        let json_token = String::from_utf8(b64_vec).unwrap();
        if let Ok(ti) = serde_json::from_str(json_token.as_str()) {
            return Some(ti)
        }

        None
    }
}
