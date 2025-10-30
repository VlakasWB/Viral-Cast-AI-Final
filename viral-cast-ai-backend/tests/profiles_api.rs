#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use reqwest::{Client, StatusCode};
use serde_json::json;

mod common;

fn ensure_base_url() {
    // Align tests with server port in .env (12000)
    std::env::set_var("BASE_URL", "http://localhost:12000");
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn profiles_flow_create_get_patch() {
    ensure_base_url();
    common::ensure_server_running().await;

    let client = Client::new();
    let token = common::register_and_login(&client).await;

    // After registration, GET my profile should be 200 (auto-created)
    let get_my_url = format!("{}/api/v1/profiles", common::base_url());
    let res = client
        .get(get_my_url.clone())
        .bearer_auth(&token)
        .send()
        .await
        .expect("get my profile resp");
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "expected 200 after auto-creation"
    );
    let get_json: serde_json::Value = res.json().await.expect("get profile json");
    assert_eq!(get_json["code"], 200);
    assert_eq!(get_json["status"], "OK");
    let profile_uuid = get_json["data"]["uuid"].as_str().expect("profile uuid");
    assert!(!profile_uuid.is_empty());

    // Patch my profile: update telp
    let patch_body = json!({ "telp": "08123456700" });
    let patch_res = client
        .patch(get_my_url.clone())
        .bearer_auth(&token)
        .json(&patch_body)
        .send()
        .await
        .expect("patch profile resp");
    assert_eq!(
        patch_res.status(),
        StatusCode::OK,
        "patch profile should be 200"
    );
    let patch_json: serde_json::Value = patch_res.json().await.expect("patch profile json");
    assert_eq!(patch_json["code"], 200);
    assert_eq!(patch_json["status"], "OK");
    assert_eq!(patch_json["data"]["telp"], "08123456700");

    // Get user profile by user UUID
    let me_url = format!("{}/api/v1/users/me", common::base_url());
    let me_res = client
        .get(me_url)
        .bearer_auth(&token)
        .send()
        .await
        .expect("get me resp");
    assert_eq!(me_res.status(), StatusCode::OK, "get me should return 200");
    let me_json: serde_json::Value = me_res.json().await.expect("me json");
    let user_uuid = me_json["data"]["uuid"].as_str().expect("user uuid");

    let get_user_url = format!("{}/api/v1/profiles/{}", common::base_url(), user_uuid);
    let get_user_res = client
        .get(get_user_url)
        .bearer_auth(&token)
        .send()
        .await
        .expect("get user profile resp");
    assert_eq!(
        get_user_res.status(),
        StatusCode::OK,
        "get user profile should be 200"
    );
    let get_user_json: serde_json::Value =
        get_user_res.json().await.expect("get user profile json");
    assert_eq!(get_user_json["code"], 200);
    assert_eq!(get_user_json["status"], "OK");
}
