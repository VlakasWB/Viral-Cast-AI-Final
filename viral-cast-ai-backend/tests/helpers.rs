#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use chrono::Utc;
use reqwest::{header::AUTHORIZATION, Client, StatusCode};
use serde_json::{json, Value};
use uuid::Uuid;

#[path = "common.rs"]
pub mod common;

pub fn ensure_base_url() {
    if std::env::var("BASE_URL").is_err() {
        std::env::set_var("BASE_URL", "http://localhost:12000");
    }
}

pub async fn create_uom(client: &Client, token: &str) -> (String, String, String) {
    let code = format!("UOM{}", &Uuid::new_v4().to_string()[..6]);
    let name = format!("Unit {}", &Uuid::new_v4().to_string()[..6]);
    let res = client
        .post(format!("{}/api/v1/uoms", common::base_url()))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({ "code": code, "name": name }))
        .send()
        .await
        .expect("create uom request");
    assert_eq!(res.status(), StatusCode::CREATED, "create uom failed");
    let json: Value = res.json().await.expect("create uom json");
    let uuid = json["data"]["uuid"].as_str().expect("uom uuid").to_string();
    (uuid, code, name)
}

pub async fn create_category(client: &Client, token: &str) -> (String, String) {
    let name = format!("Category {}", &Uuid::new_v4().to_string()[..8]);
    let res = client
        .post(format!("{}/api/v1/categories", common::base_url()))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({ "name": name }))
        .send()
        .await
        .expect("create category request");
    assert_eq!(res.status(), StatusCode::CREATED, "create category failed");
    let json: Value = res.json().await.expect("create category json");
    let uuid = json["data"]["uuid"]
        .as_str()
        .expect("category uuid")
        .to_string();
    (uuid, name)
}

pub async fn create_ingredient(
    client: &Client,
    token: &str,
    base_uom_uuid: &str,
) -> (String, String) {
    let name = format!("Ingredient {}", &Uuid::new_v4().to_string()[..8]);
    let res = client
        .post(format!("{}/api/v1/ingredient-catalog", common::base_url()))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "name": name,
            "base_uom_uuid": base_uom_uuid,
            "minimum_stock": 5.0,
            "shelf_life_days": 7
        }))
        .send()
        .await
        .expect("create ingredient request");
    assert_eq!(
        res.status(),
        StatusCode::CREATED,
        "create ingredient failed"
    );
    let json: Value = res.json().await.expect("create ingredient json");
    let uuid = json["data"]["uuid"]
        .as_str()
        .expect("ingredient uuid")
        .to_string();
    (uuid, name)
}

pub async fn create_recipe_set(client: &Client, token: &str) -> (String, String) {
    let name = format!("Recipe {}", &Uuid::new_v4().to_string()[..8]);
    let res = client
        .post(format!("{}/api/recipe-sets", common::base_url()))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "name": name,
            "yield_quantity": 1.0,
            "is_active": true
        }))
        .send()
        .await
        .expect("create recipe set request");
    assert_eq!(
        res.status(),
        StatusCode::CREATED,
        "create recipe set failed"
    );
    let json: Value = res.json().await.expect("create recipe set json");
    let uuid = json["data"]["recipe_set"]["uuid"]
        .as_str()
        .expect("recipe set uuid")
        .to_string();
    (uuid, name)
}

pub async fn create_ingredient_market_price(
    client: &Client,
    token: &str,
    ingredient_uuid: &str,
) -> Value {
    let res = client
        .post(format!(
            "{}/api/v1/ingredient-market-prices",
            common::base_url()
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "ingredient_catalog_uuid": ingredient_uuid,
            "name": format!("Market {}", &Uuid::new_v4().to_string()[..6]),
            "price": 12.5,
            "effective_at": Utc::now().timestamp_millis()
        }))
        .send()
        .await
        .expect("create ingredient market price request");
    assert_eq!(
        res.status(),
        StatusCode::CREATED,
        "create ingredient market price failed"
    );
    res.json()
        .await
        .expect("create ingredient market price json")
}

pub async fn create_ingredient_stock_move(
    client: &Client,
    token: &str,
    ingredient_uuid: &str,
) -> Value {
    let now = Utc::now().timestamp_millis();
    let res = client
        .post(format!(
            "{}/api/v1/ingredient-stock-moves",
            common::base_url()
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "ingredient_catalog_uuid": ingredient_uuid,
            "quantity": 4.5,
            "price": 11.0,
            "price_updated_at": now,
            "effective_at": now,
            "expiry_at": now + 86_400_000,
            "ref_type": "PURCHASE",
            "ref_uuid": Uuid::new_v4()
        }))
        .send()
        .await
        .expect("create ingredient stock move request");
    assert_eq!(
        res.status(),
        StatusCode::CREATED,
        "create ingredient stock move failed"
    );
    res.json().await.expect("create ingredient stock move json")
}

pub async fn create_recipe_item(
    client: &Client,
    token: &str,
    recipe_set_uuid: &str,
    ingredient_stock_uuid: &str,
) -> Value {
    let res = client
        .post(format!("{}/api/recipe-items", common::base_url()))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "recipe_sets_uuid": recipe_set_uuid,
            "ingredient_stocks_uuid": ingredient_stock_uuid,
            "quantity": 2.0,
            "waste_percent": 0.05
        }))
        .send()
        .await
        .expect("create recipe item request");
    assert_eq!(
        res.status(),
        StatusCode::CREATED,
        "create recipe item failed"
    );
    res.json().await.expect("create recipe item json")
}

pub async fn create_product(
    client: &Client,
    token: &str,
    category_uuid: &str,
    recipe_set_uuid: Option<&str>,
    price: f64,
) -> Value {
    let sku = format!("SKU{}", &Uuid::new_v4().to_string()[..6]);
    let mut body = json!({
        "category_uuid": category_uuid,
        "name": format!("Product {}", &Uuid::new_v4().to_string()[..8]),
        "price": price,
        "status": "ACTIVE",
        "sku": sku
    });
    if let Some(recipe_uuid) = recipe_set_uuid {
        body["recipe_sets_uuid"] = json!(recipe_uuid);
    }
    let res = client
        .post(format!("{}/api/v1/products", common::base_url()))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .expect("create product request");
    assert_eq!(res.status(), StatusCode::CREATED, "create product failed");
    res.json().await.expect("create product json")
}

pub async fn create_order(client: &Client, token: &str, product_uuid: &str) -> Value {
    let order_no = format!("ORD{}", &Uuid::new_v4().to_string()[..8]);
    let subtotal = 100.0;
    let res = client
        .post(format!("{}/api/v1/orders", common::base_url()))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "order_no": order_no,
            "subtotal": subtotal,
            "discount": 0,
            "tax": 0,
            "total": subtotal,
            "net_profit": 25.0,
            "items": [
                {
                    "product_uuid": product_uuid,
                    "qty": 2,
                    "unit_price": 50.0,
                    "unit_cost": 25.0,
                    "line_total": 100.0
                }
            ]
        }))
        .send()
        .await
        .expect("create order request");
    assert_eq!(res.status(), StatusCode::CREATED, "create order failed");
    res.json().await.expect("create order json")
}

pub async fn create_payment(client: &Client, token: &str, order_uuid: &str) -> Value {
    let res = client
        .post(format!("{}/api/v1/payments", common::base_url()))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "order_uuid": order_uuid,
            "method": "CASH",
            "amount": 100.0,
            "paid_at": Utc::now().timestamp_millis(),
            "external_ref": format!("PAY{}", &Uuid::new_v4().to_string()[..6])
        }))
        .send()
        .await
        .expect("create payment request");
    assert_eq!(res.status(), StatusCode::CREATED, "create payment failed");
    res.json().await.expect("create payment json")
}
