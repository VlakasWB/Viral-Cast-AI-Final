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
async fn categories_crud_flow() {
    common::ensure_server_running().await;
    let client = Client::new();
    let access_token = common::register_and_login(&client).await;

    // Create Category 1
    let name1 = format!("Category {}", &uuid::Uuid::new_v4().to_string()[..8]);
    let create_res1 = client
        .post(format!("{}/api/v1/categories", common::base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({"name": name1}))
        .send()
        .await
        .expect("create category 1 response");
    assert_eq!(
        create_res1.status(),
        StatusCode::CREATED,
        "create category should be 201"
    );
    let create_json1: serde_json::Value = create_res1.json().await.expect("create category 1 json");
    let cat1_uuid = create_json1["data"]["uuid"]
        .as_str()
        .expect("cat1 uuid")
        .to_string();

    // Get Category 1
    let get_res1 = client
        .get(format!(
            "{}/api/v1/categories/{}",
            common::base_url(),
            cat1_uuid
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("get category 1 response");
    assert_eq!(
        get_res1.status(),
        StatusCode::OK,
        "get category should be 200"
    );
    let get_json1: serde_json::Value = get_res1.json().await.expect("get category json");
    assert_eq!(get_json1["data"]["name"].as_str(), Some(name1.as_str()));

    // List Categories
    let list_res = client
        .get(format!("{}/api/v1/categories", common::base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("list categories response");
    assert_eq!(
        list_res.status(),
        StatusCode::OK,
        "list categories should be 200"
    );
    let list_json: serde_json::Value = list_res.json().await.expect("list categories json");
    assert!(list_json["data"].is_array(), "data should be an array");

    // Update Category 1
    let name1_new = format!("{} Updated", name1);
    let update_res1 = client
        .patch(format!(
            "{}/api/v1/categories/{}",
            common::base_url(),
            cat1_uuid
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({"name": name1_new}))
        .send()
        .await
        .expect("update category 1 response");
    assert_eq!(
        update_res1.status(),
        StatusCode::OK,
        "update category should be 200"
    );
    let update_json1: serde_json::Value = update_res1.json().await.expect("update category json");
    assert_eq!(
        update_json1["data"]["name"].as_str(),
        Some(name1_new.as_str())
    );

    // Create Category 2
    let name2 = format!("Category {}", &uuid::Uuid::new_v4().to_string()[..8]);
    let create_res2 = client
        .post(format!("{}/api/v1/categories", common::base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({"name": name2}))
        .send()
        .await
        .expect("create category 2 response");
    assert_eq!(
        create_res2.status(),
        StatusCode::CREATED,
        "create category 2 should be 201"
    );
    let create_json2: serde_json::Value = create_res2.json().await.expect("create category 2 json");
    let cat2_uuid = create_json2["data"]["uuid"]
        .as_str()
        .expect("cat2 uuid")
        .to_string();

    // Create duplicate -> 409
    let dup_res = client
        .post(format!("{}/api/v1/categories", common::base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({"name": name1_new}))
        .send()
        .await
        .expect("duplicate category create response");
    assert_eq!(
        dup_res.status(),
        StatusCode::CONFLICT,
        "duplicate create should be 409"
    );

    // Update Category 2 to duplicate -> 409
    let update_dup_res = client
        .patch(format!(
            "{}/api/v1/categories/{}",
            common::base_url(),
            cat2_uuid
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({"name": name1_new}))
        .send()
        .await
        .expect("update category 2 to duplicate response");
    assert_eq!(
        update_dup_res.status(),
        StatusCode::CONFLICT,
        "duplicate update should be 409"
    );

    // Delete Category 1
    let delete_res1 = client
        .delete(format!(
            "{}/api/v1/categories/{}",
            common::base_url(),
            cat1_uuid
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("delete category 1 response");
    assert_eq!(
        delete_res1.status(),
        StatusCode::OK,
        "delete category should be 200"
    );

    // Ensure Category 1 not found
    let get_after_delete1 = client
        .get(format!(
            "{}/api/v1/categories/{}",
            common::base_url(),
            cat1_uuid
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("get category 1 after delete");
    assert_eq!(
        get_after_delete1.status(),
        StatusCode::NOT_FOUND,
        "category should be 404 after delete"
    );

    // Cleanup: delete category 2
    let _ = client
        .delete(format!(
            "{}/api/v1/categories/{}",
            common::base_url(),
            cat2_uuid
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await;
}
