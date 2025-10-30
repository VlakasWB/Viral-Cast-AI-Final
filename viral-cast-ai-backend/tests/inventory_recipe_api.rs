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
async fn inventory_and_recipe_endpoints_flow() {
    ensure_base_url();
    common::ensure_server_running().await;
    let client = Client::new();
    let token = common::register_and_login(&client).await;

    // Create base data
    let (uom_uuid, _, _) = helpers::create_uom(&client, &token).await;
    let (ingredient_uuid, _) = helpers::create_ingredient(&client, &token, &uom_uuid).await;
    let stock_move_json =
        helpers::create_ingredient_stock_move(&client, &token, &ingredient_uuid).await;
    let stock_move_uuid = stock_move_json["data"]["uuid"]
        .as_str()
        .expect("stock move uuid")
        .to_string();

    let stock_list = client
        .get(format!(
            "{}/api/v1/ingredient-stocks?ingredient_catalog_uuid={}",
            common::base_url(),
            ingredient_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("list ingredient stocks");
    assert_eq!(stock_list.status(), StatusCode::OK);
    let stock_list_json: Value = stock_list.json().await.expect("list stock json");
    let stock_array = stock_list_json["data"]
        .as_array()
        .expect("stock array available");
    assert!(
        !stock_array.is_empty(),
        "stock array should contain at least one item"
    );
    let stock_uuid = stock_array[0]["uuid"]
        .as_str()
        .expect("ingredient stock uuid")
        .to_string();

    let market_price_json =
        helpers::create_ingredient_market_price(&client, &token, &ingredient_uuid).await;
    let market_price_uuid = market_price_json["data"]["uuid"]
        .as_str()
        .expect("market price uuid")
        .to_string();

    let (recipe_set_uuid, _) = helpers::create_recipe_set(&client, &token).await;
    let recipe_item_json =
        helpers::create_recipe_item(&client, &token, &recipe_set_uuid, &stock_uuid).await;
    let recipe_item_uuid = recipe_item_json["data"]["recipe_item"]["uuid"]
        .as_str()
        .expect("recipe item uuid")
        .to_string();

    // Ingredient stock endpoints
    let get_stock = client
        .get(format!(
            "{}/api/v1/ingredient-stocks/{}",
            common::base_url(),
            stock_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("get ingredient stock");
    assert_eq!(get_stock.status(), StatusCode::OK);

    let update_stock = client
        .put(format!(
            "{}/api/v1/ingredient-stocks/{}",
            common::base_url(),
            stock_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({ "total_quantity": 20, "total_value": 200 }))
        .send()
        .await
        .expect("update ingredient stock");
    assert_eq!(update_stock.status(), StatusCode::OK);
    let update_stock_json: Value = update_stock.json().await.expect("update stock json");
    assert_eq!(
        update_stock_json["total_quantity"].as_i64(),
        Some(20),
        "stock quantity should update"
    );

    let list_stocks = client
        .get(format!(
            "{}/api/v1/ingredient-stocks?ingredient_catalog_uuid={}",
            common::base_url(),
            ingredient_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("list ingredient stocks");
    assert_eq!(list_stocks.status(), StatusCode::OK);
    let list_stocks_json: Value = list_stocks.json().await.expect("list stock json");
    assert!(list_stocks_json["data"].is_array());

    // Ingredient market price endpoints
    let get_market = client
        .get(format!(
            "{}/api/v1/ingredient-market-prices/{}",
            common::base_url(),
            market_price_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("get market price");
    assert_eq!(get_market.status(), StatusCode::OK);

    let update_market = client
        .put(format!(
            "{}/api/v1/ingredient-market-prices/{}",
            common::base_url(),
            market_price_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({ "price": 15.75 }))
        .send()
        .await
        .expect("update market price");
    assert_eq!(update_market.status(), StatusCode::OK);
    let update_market_json: Value = update_market.json().await.expect("update market json");
    assert_eq!(
        update_market_json["data"]["price"].as_f64(),
        Some(15.75),
        "market price should update"
    );

    // Ingredient stock move endpoints
    let get_move = client
        .get(format!(
            "{}/api/v1/ingredient-stock-moves/{}",
            common::base_url(),
            stock_move_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("get stock move");
    assert_eq!(get_move.status(), StatusCode::OK);

    let update_move = client
        .patch(format!(
            "{}/api/v1/ingredient-stock-moves/{}",
            common::base_url(),
            stock_move_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({ "quantity": 6.0 }))
        .send()
        .await
        .expect("update stock move");
    assert_eq!(update_move.status(), StatusCode::OK);
    let update_move_json: Value = update_move.json().await.expect("update move json");
    assert_eq!(
        update_move_json["data"]["quantity"].as_f64(),
        Some(6.0),
        "stock move quantity should update"
    );

    // Recipe set endpoints
    let recipe_set_get = client
        .get(format!(
            "{}/api/recipe-sets/{}",
            common::base_url(),
            recipe_set_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("get recipe set");
    assert_eq!(recipe_set_get.status(), StatusCode::OK);

    let recipe_set_update = client
        .put(format!(
            "{}/api/recipe-sets/{}",
            common::base_url(),
            recipe_set_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({ "yield_quantity": 2.5, "is_active": true }))
        .send()
        .await
        .expect("update recipe set");
    assert_eq!(recipe_set_update.status(), StatusCode::OK);
    let recipe_set_update_json: Value = recipe_set_update
        .json()
        .await
        .expect("update recipe set json");
    assert_eq!(
        recipe_set_update_json["data"]["recipe_set"]["yield_quantity"].as_f64(),
        Some(2.5)
    );

    // Recipe item endpoints
    let recipe_item_get = client
        .get(format!(
            "{}/api/recipe-items/{}",
            common::base_url(),
            recipe_item_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("get recipe item");
    assert_eq!(recipe_item_get.status(), StatusCode::OK);

    let recipe_item_update = client
        .put(format!(
            "{}/api/recipe-items/{}",
            common::base_url(),
            recipe_item_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({ "quantity": 3.0 }))
        .send()
        .await
        .expect("update recipe item");
    assert_eq!(recipe_item_update.status(), StatusCode::OK);
    let recipe_item_update_json: Value = recipe_item_update
        .json()
        .await
        .expect("update recipe item json");
    assert_eq!(
        recipe_item_update_json["data"]["recipe_item"]["quantity"].as_f64(),
        Some(3.0)
    );

    let recipe_items_list = client
        .get(format!(
            "{}/api/recipe-items?recipe_sets_uuid={}",
            common::base_url(),
            recipe_set_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("list recipe items");
    assert_eq!(recipe_items_list.status(), StatusCode::OK);
    let recipe_items_list_json: Value = recipe_items_list
        .json()
        .await
        .expect("recipe items list json");
    assert!(recipe_items_list_json["data"]["recipe_items"].is_array());
}
