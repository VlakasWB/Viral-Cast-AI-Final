#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use reqwest::{Client, StatusCode};
use serde_json::json;

mod common;

fn ensure_base_url() {
    std::env::set_var("BASE_URL", "http://localhost:12000");
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn stores_flow_create_get_put_patch() {
    ensure_base_url();
    common::ensure_server_running().await;

    let client = Client::new();
    let token = common::register_and_login(&client).await;

    // Create store (minimal payload)
    let create_url = format!("{}/api/v1/stores", common::base_url());
    let create_body = json!({ "name": "Toko Jaya" });
    let create_res = client
        .post(create_url.clone())
        .bearer_auth(&token)
        .json(&create_body)
        .send()
        .await
        .expect("create store resp");
    assert_eq!(
        create_res.status(),
        StatusCode::CREATED,
        "create store should be 201"
    );
    let create_json: serde_json::Value = create_res.json().await.expect("create store json");
    assert_eq!(create_json["code"], 201);
    assert_eq!(create_json["status"], "success");
    let store_uuid = create_json["data"]["uuid"].as_str().expect("store uuid");

    // Get store
    let get_url = format!("{}/api/v1/stores/{}", common::base_url(), store_uuid);
    let get_res = client
        .get(get_url.clone())
        .bearer_auth(&token)
        .send()
        .await
        .expect("get store resp");
    assert_eq!(get_res.status(), StatusCode::OK, "get store should be 200");
    let get_json: serde_json::Value = get_res.json().await.expect("get store json");
    assert_eq!(get_json["code"], 200);
    assert_eq!(get_json["status"], "success");
    assert_eq!(get_json["data"]["name"], "Toko Jaya");

    // Update store (PUT): change name and telp
    let put_body = json!({
        "name": "Toko Jaya Baru",
        "telp": "021-1234567"
    });
    let put_res = client
        .put(get_url.clone())
        .bearer_auth(&token)
        .json(&put_body)
        .send()
        .await
        .expect("put store resp");
    assert_eq!(
        put_res.status(),
        StatusCode::OK,
        "update store (PUT) should be 200"
    );
    let put_json: serde_json::Value = put_res.json().await.expect("put store json");
    assert_eq!(put_json["code"], 200);
    assert_eq!(put_json["status"], "success");
    assert_eq!(put_json["data"]["name"], "Toko Jaya Baru");
    assert_eq!(put_json["data"]["telp"], "021-1234567");

    // Partial update (PATCH): add instagram
    let patch_body = json!({ "instagram": "@tokojaya" });
    let patch_res = client
        .patch(get_url.clone())
        .bearer_auth(&token)
        .json(&patch_body)
        .send()
        .await
        .expect("patch store resp");
    assert_eq!(
        patch_res.status(),
        StatusCode::OK,
        "patch store should be 200"
    );
    let patch_json: serde_json::Value = patch_res.json().await.expect("patch store json");
    assert_eq!(patch_json["code"], 200);
    assert_eq!(patch_json["status"], "success");
    assert_eq!(patch_json["data"]["instagram"], "@tokojaya");
}
