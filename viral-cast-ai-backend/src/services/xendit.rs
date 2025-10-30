use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct XenditQrCodeResponse {
    pub id: String,
    pub reference_id: String,
    #[serde(rename = "type")]
    pub qr_type: String, // typically "DYNAMIC"
    pub amount: i64,
    pub currency: String, // e.g., "IDR"
    pub status: String,   // e.g., "ACTIVE", "COMPLETED"
    pub qr_string: Option<String>,
    pub qr_code_url: Option<String>,
    pub expires_at: Option<String>,
}

pub async fn create_qr_code(
    client: &Client,
    secret_key: &str,
    reference_id: &str,
    amount: i64,
    callback_url: Option<&str>,
) -> anyhow::Result<XenditQrCodeResponse> {
    let body = json!({
        "reference_id": reference_id,
        "type": "DYNAMIC",
        "amount": amount,
        "currency": "IDR",
        "callback_url": callback_url,
    });

    let resp = client
        .post("https://api.xendit.co/qr_codes")
        .basic_auth(secret_key, Some(""))
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    let parsed = resp.json::<XenditQrCodeResponse>().await?;
    Ok(parsed)
}

pub async fn get_qr_code(
    client: &Client,
    secret_key: &str,
    qr_id: &str,
) -> anyhow::Result<XenditQrCodeResponse> {
    let url = format!("https://api.xendit.co/qr_codes/{}", qr_id);
    let resp = client
        .get(url)
        .basic_auth(secret_key, Some(""))
        .send()
        .await?
        .error_for_status()?;

    let parsed = resp.json::<XenditQrCodeResponse>().await?;
    Ok(parsed)
}

/// Validates a Xendit callback token header against expected tokens.
/// Returns true if token from header matches any expected token.
pub fn validate_callback_token(token_header: Option<&str>, expected_tokens: &[String]) -> bool {
    match token_header {
        Some(incoming) => expected_tokens.iter().any(|t| t == incoming),
        None => false,
    }
}
