#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use reqwest::{Client, StatusCode};
use serde_json::json;

#[path = "common.rs"]
mod common;

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn uoms_crud_flow() {
    common::ensure_server_running().await;
    let client = Client::new();
    let access_token = common::register_and_login(&client).await;

    // Create UOM
    let code = format!("KG{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let name = format!("Kilogram {}", &uuid::Uuid::new_v4().to_string()[..4]);
    let create_res = client
        .post(format!("{}/api/v1/uoms", common::base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({"code": code, "name": name}))
        .send()
        .await
        .expect("create uom response");
    assert_eq!(
        create_res.status(),
        StatusCode::CREATED,
        "create uom should be 201"
    );
    let create_json: serde_json::Value = create_res.json().await.expect("create uom json");
    let uom_uuid = create_json["data"]["uuid"]
        .as_str()
        .expect("uom uuid")
        .to_string();

    // Get UOM by id
    let get_res = client
        .get(format!("{}/api/v1/uoms/{}", common::base_url(), uom_uuid))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("get uom response");
    assert_eq!(get_res.status(), StatusCode::OK, "get uom should be 200");
    let get_json: serde_json::Value = get_res.json().await.expect("get uom json");
    assert_eq!(get_json["data"]["code"].as_str(), Some(code.as_str()));
    assert_eq!(get_json["data"]["name"].as_str(), Some(name.as_str()));

    // List UOMs
    let list_res = client
        .get(format!("{}/api/v1/uoms", common::base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("list uoms response");
    assert_eq!(list_res.status(), StatusCode::OK, "list uoms should be 200");
    let list_json: serde_json::Value = list_res.json().await.expect("list uoms json");
    assert!(list_json["data"].is_array(), "data should be an array");

    // Update UOM (PATCH)
    let new_name = format!("{} Updated", name);
    let update_res = client
        .patch(format!("{}/api/v1/uoms/{}", common::base_url(), uom_uuid))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({"name": new_name}))
        .send()
        .await
        .expect("update uom response");
    assert_eq!(
        update_res.status(),
        StatusCode::OK,
        "update uom should be 200"
    );
    let update_json: serde_json::Value = update_res.json().await.expect("update uom json");
    assert_eq!(
        update_json["data"]["name"].as_str(),
        Some(new_name.as_str())
    );

    // Delete UOM
    let delete_res = client
        .delete(format!("{}/api/v1/uoms/{}", common::base_url(), uom_uuid))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("delete uom response");
    assert_eq!(
        delete_res.status(),
        StatusCode::OK,
        "delete uom should be 200"
    );

    // Ensure UOM not found after delete
    let get_after_delete = client
        .get(format!("{}/api/v1/uoms/{}", common::base_url(), uom_uuid))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("get uom after delete");
    assert_eq!(
        get_after_delete.status(),
        StatusCode::NOT_FOUND,
        "uom should be 404 after delete"
    );
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn uoms_search_flow() {
    common::ensure_server_running().await;
    let client = Client::new();
    let access_token = common::register_and_login(&client).await;

    // Create test UOMs with unique identifiers
    let unique_id = &uuid::Uuid::new_v4().to_string()[..6];

    // Create first UOM
    let code1 = format!("TEST{}", unique_id);
    let name1 = format!("Test Unit {}", unique_id);
    client
        .post(format!("{}/api/v1/uoms", common::base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({"code": code1, "name": name1}))
        .send()
        .await
        .expect("create first uom");

    // Create second UOM
    let code2 = format!("UNIT{}", unique_id);
    let name2 = format!("Another Test {}", unique_id);
    client
        .post(format!("{}/api/v1/uoms", common::base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({"code": code2, "name": name2}))
        .send()
        .await
        .expect("create second uom");

    // Test search by general term
    let search_res = client
        .get(format!(
            "{}/api/v1/uoms?search={}",
            common::base_url(),
            unique_id
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("search uoms response");
    assert_eq!(
        search_res.status(),
        StatusCode::OK,
        "search uoms should be 200"
    );
    let search_json: serde_json::Value = search_res.json().await.expect("search uoms json");

    // Verify search results contain both UOMs
    let items = search_json["data"]["items"]
        .as_array()
        .expect("items should be an array");
    assert!(items.len() >= 2, "search should return at least 2 items");

    // Verify pagination info exists
    assert!(
        search_json["data"]["pagination"].is_object(),
        "pagination info should exist"
    );
    assert!(
        search_json["data"]["pagination"]["total_items"].is_number(),
        "total_items should exist"
    );
    assert!(
        search_json["data"]["pagination"]["total_pages"].is_number(),
        "total_pages should exist"
    );
    assert!(
        search_json["data"]["pagination"]["current_page"].is_number(),
        "current_page should exist"
    );
    assert!(
        search_json["data"]["pagination"]["items_per_page"].is_number(),
        "items_per_page should exist"
    );

    // Test search by specific code
    let code_search_res = client
        .get(format!("{}/api/v1/uoms?code={}", common::base_url(), code1))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("code search response");
    assert_eq!(
        code_search_res.status(),
        StatusCode::OK,
        "code search should be 200"
    );
    let code_search_json: serde_json::Value =
        code_search_res.json().await.expect("code search json");

    // Verify code search results
    let code_items = code_search_json["data"]["items"]
        .as_array()
        .expect("code items should be an array");
    assert!(
        code_items.len() > 0,
        "code search should return at least 1 item"
    );
    assert_eq!(
        code_items[0]["code"].as_str(),
        Some(code1.as_str()),
        "code search should return correct item"
    );

    // Test search by specific name
    let name_search_res = client
        .get(format!(
            "{}/api/v1/uoms?name={}",
            common::base_url(),
            "Another"
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("name search response");
    assert_eq!(
        name_search_res.status(),
        StatusCode::OK,
        "name search should be 200"
    );
    let name_search_json: serde_json::Value =
        name_search_res.json().await.expect("name search json");

    // Verify name search results
    let name_items = name_search_json["data"]["items"]
        .as_array()
        .expect("name items should be an array");
    assert!(
        name_items.len() > 0,
        "name search should return at least 1 item"
    );

    // Test pagination
    let pagination_res = client
        .get(format!("{}/api/v1/uoms?page=1&limit=5", common::base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("pagination response");
    assert_eq!(
        pagination_res.status(),
        StatusCode::OK,
        "pagination should be 200"
    );
    let pagination_json: serde_json::Value = pagination_res.json().await.expect("pagination json");

    // Verify pagination results
    let pagination_items = pagination_json["data"]["items"]
        .as_array()
        .expect("pagination items should be an array");
    assert!(
        pagination_items.len() <= 5,
        "pagination should return at most 5 items"
    );
    assert_eq!(
        pagination_json["data"]["pagination"]["items_per_page"].as_i64(),
        Some(5),
        "items_per_page should be 5"
    );
    assert_eq!(
        pagination_json["data"]["pagination"]["current_page"].as_i64(),
        Some(1),
        "current_page should be 1"
    );
}
