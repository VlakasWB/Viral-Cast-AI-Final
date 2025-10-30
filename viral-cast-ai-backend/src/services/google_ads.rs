use reqwest::Client;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone)]
pub struct GoogleAdsClient {
    client: Client,
}

#[derive(Debug)]
pub enum GoogleAdsError {
    MissingEnv(String),
    Http(String),
    Serde(String),
}

impl std::fmt::Display for GoogleAdsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoogleAdsError::MissingEnv(name) => write!(f, "Missing env var {}", name),
            GoogleAdsError::Http(msg) => write!(f, "HTTP error: {}", msg),
            GoogleAdsError::Serde(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for GoogleAdsError {}

impl GoogleAdsClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn get_access_token(&self) -> Result<String, GoogleAdsError> {
        let client_id = env::var("GOOGLE_ADS_CLIENT_ID")
            .map_err(|_| GoogleAdsError::MissingEnv("GOOGLE_ADS_CLIENT_ID".into()))?;
        let client_secret = env::var("GOOGLE_ADS_CLIENT_SECRET")
            .map_err(|_| GoogleAdsError::MissingEnv("GOOGLE_ADS_CLIENT_SECRET".into()))?;
        let refresh_token = env::var("GOOGLE_ADS_REFRESH_TOKEN")
            .map_err(|_| GoogleAdsError::MissingEnv("GOOGLE_ADS_REFRESH_TOKEN".into()))?;

        let params = [
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token".to_string()),
        ];

        let resp = self
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| GoogleAdsError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let txt = resp.text().await.unwrap_or_default();
            return Err(GoogleAdsError::Http(format!(
                "OAuth token failed: {} - {}",
                status, txt
            )));
        }

        #[derive(Deserialize)]
        struct TokenResp {
            access_token: String,
        }

        let tr: TokenResp = resp
            .json()
            .await
            .map_err(|e| GoogleAdsError::Serde(e.to_string()))?;
        Ok(tr.access_token)
    }

    pub async fn search(
        &self,
        customer_id: Option<String>,
        query: String,
        page_size: Option<u32>,
    ) -> Result<serde_json::Value, GoogleAdsError> {
        let developer_token = env::var("GOOGLE_ADS_DEVELOPER_TOKEN")
            .map_err(|_| GoogleAdsError::MissingEnv("GOOGLE_ADS_DEVELOPER_TOKEN".into()))?;
        let login_customer_id = env::var("GOOGLE_ADS_LOGIN_CUSTOMER_ID").ok();
        let cust_id = match customer_id.or_else(|| env::var("GOOGLE_ADS_CUSTOMER_ID").ok()) {
            Some(v) => v,
            None => return Err(GoogleAdsError::MissingEnv("GOOGLE_ADS_CUSTOMER_ID".into())),
        };
        let token = self.get_access_token().await?;

        let mut req = self
            .client
            .post(format!(
                "https://googleads.googleapis.com/v14/customers/{}/googleAds:search",
                cust_id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .header("developer-token", developer_token)
            .header("Content-Type", "application/json");

        if let Some(login_id) = login_customer_id {
            if !login_id.trim().is_empty() {
                req = req.header("login-customer-id", login_id);
            }
        }

        let body = serde_json::json!({
            "query": query,
            "pageSize": page_size.unwrap_or(100),
        });

        let resp = req
            .json(&body)
            .send()
            .await
            .map_err(|e| GoogleAdsError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let txt = resp.text().await.unwrap_or_default();
            return Err(GoogleAdsError::Http(format!(
                "Search failed: {} - {}",
                status, txt
            )));
        }

        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| GoogleAdsError::Serde(e.to_string()))?;
        Ok(v)
    }

    pub async fn list_campaigns(
        &self,
        customer_id: Option<String>,
    ) -> Result<serde_json::Value, GoogleAdsError> {
        let query =
            "SELECT campaign.id, campaign.name, campaign.status FROM campaign ORDER BY campaign.id";
        self.search(customer_id, query.to_string(), Some(100)).await
    }
}
