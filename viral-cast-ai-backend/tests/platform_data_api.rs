#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use reqwest::{header::AUTHORIZATION, Client, StatusCode};
use serde_json::{json, Value};

mod helpers;
use helpers::{common, ensure_base_url};

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn roles_regions_weather_and_rag_endpoints() {
    ensure_base_url();
    common::ensure_server_running().await;
    let client = Client::new();
    let token = common::register_and_login(&client).await;

    // Roles require authentication
    let roles_unauth = client
        .get(format!("{}/api/v1/roles", common::base_url()))
        .send()
        .await
        .expect("roles without auth");
    assert_eq!(roles_unauth.status(), StatusCode::UNAUTHORIZED);

    let roles_res = client
        .get(format!("{}/api/v1/roles", common::base_url()))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("roles list");
    assert_eq!(roles_res.status(), StatusCode::OK);
    let roles_json: Value = roles_res.json().await.expect("roles json");
    assert!(roles_json["data"].is_array(), "roles data should be array");

    let role_not_found = client
        .get(format!("{}/api/v1/roles/{}", common::base_url(), 999_999))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("role not found");
    assert_eq!(role_not_found.status(), StatusCode::NOT_FOUND);

    // Regions endpoints
    let provinces_res = client
        .get(format!(
            "{}/api/v1/regions/provinces?limit=5&offset=0",
            common::base_url()
        ))
        .send()
        .await
        .expect("provinces list");
    assert_eq!(provinces_res.status(), StatusCode::OK);
    let provinces_json: Value = provinces_res.json().await.expect("provinces json");
    assert!(
        provinces_json["data"].is_array(),
        "provinces data should be array"
    );

    let province_detail_missing = client
        .get(format!(
            "{}/api/v1/regions/provinces/detail",
            common::base_url()
        ))
        .send()
        .await
        .expect("province detail missing");
    assert_eq!(province_detail_missing.status(), StatusCode::OK);
    let province_detail_json: Value = province_detail_missing
        .json()
        .await
        .expect("province detail json");
    assert_eq!(
        province_detail_json["success"].as_bool(),
        Some(false),
        "province detail without params should fail"
    );

    // Weather BMKG endpoints
    let invalid_prediction = client
        .get(format!(
            "{}/api/v1/weather_bmkg/prediction?region_code=invalid",
            common::base_url()
        ))
        .send()
        .await
        .expect("invalid weather prediction");
    assert_eq!(invalid_prediction.status(), StatusCode::OK);
    let invalid_prediction_json: Value = invalid_prediction.json().await.expect("prediction json");
    assert_eq!(
        invalid_prediction_json["success"].as_bool(),
        Some(false),
        "invalid region should return success=false"
    );

    let prioritized_res = client
        .get(format!(
            "{}/api/v1/weather_bmkg/priorities",
            common::base_url()
        ))
        .send()
        .await
        .expect("prioritized regions");
    assert_eq!(prioritized_res.status(), StatusCode::OK);
    let prioritized_json: Value = prioritized_res.json().await.expect("prioritized json");
    assert!(
        prioritized_json.is_array(),
        "prioritized regions should be array"
    );

    // RAG configuration endpoints
    let rag_config = client
        .get(format!("{}/api/rag/config", common::base_url()))
        .send()
        .await
        .expect("get rag config");
    assert_eq!(rag_config.status(), StatusCode::OK);
    let rag_config_json: Value = rag_config.json().await.expect("rag config json");
    assert_eq!(rag_config_json["status"], "OK");

    let rag_update = client
        .put(format!("{}/api/rag/config", common::base_url()))
        .json(&json!({
            "max_results": 15,
            "enable_reranking": true
        }))
        .send()
        .await
        .expect("update rag config");
    assert_eq!(rag_update.status(), StatusCode::OK);
    let rag_update_json: Value = rag_update.json().await.expect("rag update json");
    assert_eq!(
        rag_update_json["data"]["max_results"].as_i64(),
        Some(15),
        "RAG max_results should update"
    );
}
