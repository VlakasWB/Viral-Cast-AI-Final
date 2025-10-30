#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use reqwest::{Client, StatusCode};
use serde_json::json;

#[path = "common.rs"]
mod common;

fn ensure_base_url() {
    // Align tests with server port configured via .env (commonly 12000)
    std::env::set_var("BASE_URL", "http://localhost:12000");
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn ingredients_crud_flow() {
    ensure_base_url();
    common::ensure_server_running().await;
    let client = Client::new();
    let access_token = common::register_and_login(&client).await;

    // Create base UOM for ingredient
    let uom_code = format!("PCS{}", &uuid::Uuid::new_v4().to_string()[..7]);
    let uom_name = format!("Pieces {}", &uuid::Uuid::new_v4().to_string()[..4]);
    let uom_res = client
        .post(format!("{}/api/v1/uoms", common::base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({"code": uom_code, "name": uom_name}))
        .send()
        .await
        .expect("create uom response for ingredient");
    assert_eq!(
        uom_res.status(),
        StatusCode::CREATED,
        "uom create should be 201"
    );
    let uom_json: serde_json::Value = uom_res.json().await.expect("uom json");
    let base_uom_uuid = uom_json["data"]["uuid"]
        .as_str()
        .expect("base uom uuid")
        .to_string();

    // Create ingredient
    let ing_name = format!("Tomato {}", &uuid::Uuid::new_v4().to_string()[..6]);
    let create_ing_res = client
        .post(format!("{}/api/v1/ingredient-catalog", common::base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({
            "name": ing_name,
            "base_uom_uuid": base_uom_uuid,
            "minimum_stock": 10.5,
            "shelf_life_days": 30
        }))
        .send()
        .await
        .expect("create ingredient response");
    assert_eq!(
        create_ing_res.status(),
        StatusCode::CREATED,
        "ingredient create should be 201"
    );
    let create_ing_json: serde_json::Value =
        create_ing_res.json().await.expect("ingredient create json");
    let ingredient_uuid = create_ing_json["data"]["uuid"]
        .as_str()
        .expect("ingredient uuid")
        .to_string();
    assert_eq!(
        create_ing_json["data"]["base_uom_uuid"]["uuid"].as_str(),
        Some(base_uom_uuid.as_str())
    );

    // Get ingredient by id
    let get_ing_res = client
        .get(format!(
            "{}/api/v1/ingredient-catalog/{}",
            common::base_url(),
            ingredient_uuid
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("get ingredient response");
    assert_eq!(
        get_ing_res.status(),
        StatusCode::OK,
        "ingredient get should be 200"
    );
    let get_ing_json: serde_json::Value = get_ing_res.json().await.expect("ingredient get json");
    assert_eq!(
        get_ing_json["data"]["name"].as_str(),
        Some(ing_name.as_str())
    );

    // Update ingredient (PUT)
    let updated_name = format!("{} Updated", ing_name);
    let update_ing_res = client
        .put(format!(
            "{}/api/v1/ingredient-catalog/{}",
            common::base_url(),
            ingredient_uuid
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .json(&json!({
            "name": updated_name,
            "minimum_stock": 20.0,
            "shelf_life_days": 45
        }))
        .send()
        .await
        .expect("update ingredient response");
    assert_eq!(
        update_ing_res.status(),
        StatusCode::OK,
        "ingredient update should be 200"
    );
    let update_ing_json: serde_json::Value =
        update_ing_res.json().await.expect("ingredient update json");
    assert_eq!(
        update_ing_json["data"]["name"].as_str(),
        Some(updated_name.as_str())
    );
    assert_eq!(
        update_ing_json["data"]["shelf_life_days"].as_i64(),
        Some(45)
    );

    // List ingredients with search filter
    let list_ing_res = client
        .get(format!(
            "{}/api/v1/ingredient-catalog?search={}",
            common::base_url(),
            ing_name.split_whitespace().next().unwrap()
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("list ingredients response");
    assert_eq!(
        list_ing_res.status(),
        StatusCode::OK,
        "ingredients list should be 200"
    );
    let list_ing_json: serde_json::Value =
        list_ing_res.json().await.expect("ingredients list json");
    assert!(
        list_ing_json["data"]["ingredient_catalog"].is_array(),
        "ingredient_catalog should be an array"
    );

    // Delete ingredient
    let delete_ing_res = client
        .delete(format!(
            "{}/api/v1/ingredient-catalog/{}",
            common::base_url(),
            ingredient_uuid
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("delete ingredient response");
    assert_eq!(
        delete_ing_res.status(),
        StatusCode::OK,
        "ingredient delete should be 200"
    );

    // Ensure ingredient not found after delete
    let get_after_delete = client
        .get(format!(
            "{}/api/v1/ingredient-catalog/{}",
            common::base_url(),
            ingredient_uuid
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .expect("get ingredient after delete");
    assert_eq!(
        get_after_delete.status(),
        StatusCode::NOT_FOUND,
        "ingredient should be 404 after delete"
    );
}
