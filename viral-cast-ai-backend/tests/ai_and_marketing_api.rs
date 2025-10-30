#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

mod helpers;
use helpers::{common, ensure_base_url};

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn ai_configuration_and_controls_endpoints() {
    ensure_base_url();
    common::ensure_server_running().await;
    let client = Client::new();

    // Fetch current configuration
    let config_res = client
        .get(format!("{}/api/ai/config", common::base_url()))
        .send()
        .await
        .expect("get ai config");
    assert_eq!(
        config_res.status(),
        StatusCode::OK,
        "get config should be 200"
    );
    let config_json: Value = config_res.json().await.expect("config json");
    assert_eq!(config_json["code"].as_u64(), Some(200));
    let original_max = config_json["data"]["max_input_length"]
        .as_u64()
        .unwrap_or(0);

    // Update configuration
    let updated_max = (original_max + 128).max(512);
    let update_res = client
        .put(format!("{}/api/ai/config", common::base_url()))
        .json(&json!({
            "input_validation_enabled": true,
            "token_limit_enabled": false,
            "max_input_length": updated_max
        }))
        .send()
        .await
        .expect("update ai config");
    assert_eq!(
        update_res.status(),
        StatusCode::OK,
        "update config should be 200"
    );
    let update_json: Value = update_res.json().await.expect("update config json");
    assert_eq!(
        update_json["data"]["max_input_length"].as_u64(),
        Some(updated_max)
    );

    // Token usage endpoints
    let usage_res = client
        .get(format!("{}/api/ai/token-usage", common::base_url()))
        .send()
        .await
        .expect("token usage");
    assert_eq!(
        usage_res.status(),
        StatusCode::OK,
        "token usage should be 200"
    );
    let usage_json: Value = usage_res.json().await.expect("usage json");
    assert!(
        usage_json["data"].is_object(),
        "usage data should be object"
    );

    let detailed_res = client
        .get(format!(
            "{}/api/ai/token-usage/detailed",
            common::base_url()
        ))
        .send()
        .await
        .expect("detailed usage");
    assert_eq!(
        detailed_res.status(),
        StatusCode::OK,
        "detailed usage should be 200"
    );
    let detailed_json: Value = detailed_res.json().await.expect("detailed json");
    assert!(
        detailed_json["data"]["average_tokens_per_request"]
            .as_f64()
            .is_some(),
        "detailed usage should include average tokens"
    );

    let history_res = client
        .get(format!("{}/api/ai/token-usage/history", common::base_url()))
        .send()
        .await
        .expect("usage history");
    assert_eq!(
        history_res.status(),
        StatusCode::OK,
        "usage history should be 200"
    );
    let history_json: Value = history_res.json().await.expect("history json");
    assert!(
        history_json["data"].is_array(),
        "token usage history should be array"
    );

    let alerts_res = client
        .get(format!("{}/api/ai/token-usage/alerts", common::base_url()))
        .send()
        .await
        .expect("usage alerts");
    assert_eq!(
        alerts_res.status(),
        StatusCode::OK,
        "usage alerts should be 200"
    );
    let alerts_json: Value = alerts_res.json().await.expect("alerts json");
    assert!(alerts_json["data"].is_array(), "alerts should be array");

    // User input controls
    let controls_res = client
        .get(format!("{}/api/ai/input-controls", common::base_url()))
        .send()
        .await
        .expect("get controls");
    assert_eq!(
        controls_res.status(),
        StatusCode::OK,
        "get input controls should be 200"
    );
    let controls_json: Value = controls_res.json().await.expect("controls json");
    assert!(
        controls_json["data"].is_object(),
        "controls data should exist"
    );

    let update_controls_res = client
        .put(format!("{}/api/ai/input-controls", common::base_url()))
        .json(&json!({
            "max_input_length": 256,
            "rate_limit_per_minute": 20,
            "blocked_keywords": ["forbidden", "restricted"],
            "required_keywords": ["tolong"]
        }))
        .send()
        .await
        .expect("update controls");
    assert_eq!(
        update_controls_res.status(),
        StatusCode::OK,
        "update controls should be 200"
    );
    let update_controls_json: Value = update_controls_res
        .json()
        .await
        .expect("update controls json");
    assert_eq!(
        update_controls_json["data"]["max_input_length"].as_u64(),
        Some(256)
    );
    assert_eq!(
        update_controls_json["data"]["blocked_keywords"]
            .as_array()
            .map(|v| v.len()),
        Some(2)
    );
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn google_ads_missing_configuration() {
    ensure_base_url();
    common::ensure_server_running().await;
    let client = Client::new();

    let campaigns_res = client
        .get(format!("{}/api/google-ads/campaigns", common::base_url()))
        .send()
        .await
        .expect("list campaigns");
    assert_eq!(
        campaigns_res.status(),
        StatusCode::BAD_REQUEST,
        "missing env should be 400"
    );
    let campaigns_text = campaigns_res.text().await.expect("campaigns text");
    assert!(
        campaigns_text.contains("Missing env var")
            || campaigns_text.contains("Missing configuration"),
        "campaigns error message should mention missing configuration"
    );

    let search_res = client
        .post(format!("{}/api/google-ads/search", common::base_url()))
        .json(&json!({
            "query": "SELECT campaign.id FROM campaign",
            "page_size": 5
        }))
        .send()
        .await
        .expect("search campaigns");
    assert_eq!(
        search_res.status(),
        StatusCode::UNAUTHORIZED,
        "search should be 401 without configuration"
    );
    let search_text = search_res.text().await.expect("search text");
    assert!(
        search_text.contains("Missing configuration"),
        "search response should mention missing configuration"
    );
}
