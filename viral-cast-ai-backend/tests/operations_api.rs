#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use reqwest::{header::AUTHORIZATION, Client, StatusCode};
use serde_json::{json, Value};
use uuid::Uuid;

mod helpers;
use helpers::{common, ensure_base_url};

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn forecast_daily_requires_auth_and_validation() {
    ensure_base_url();
    common::ensure_server_running().await;
    let client = Client::new();

    // Unauthorized access should be rejected
    let unauth_get = client
        .get(format!("{}/api/forecast-daily", common::base_url()))
        .send()
        .await
        .expect("unauthorized forecast get");
    assert_eq!(unauth_get.status(), StatusCode::UNAUTHORIZED);

    let unauth_post = client
        .post(format!("{}/api/forecast-daily", common::base_url()))
        .send()
        .await
        .expect("unauthorized forecast create");
    assert_eq!(unauth_post.status(), StatusCode::UNAUTHORIZED);

    // Invalid payload should trigger validation errors
    let token = common::register_and_login(&client).await;
    let invalid_body = json!({
        "product_uuid": Uuid::new_v4(),
        "date_ts": "2019-01-01",
        "method": "invalid_method",
        "window_size": 0,
        "params": { "alpha": 2.0 },
        "forecast_qty": -10.0,
        "conf_low": -1.0,
        "conf_high": 1_000_001.0,
        "mae": 200.0,
        "mape": 150.0
    });
    let invalid_res = client
        .post(format!("{}/api/forecast-daily", common::base_url()))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&invalid_body)
        .send()
        .await
        .expect("forecast validation");
    assert_eq!(invalid_res.status(), StatusCode::BAD_REQUEST);
    let invalid_json: Value = invalid_res.json().await.expect("invalid json");
    assert_eq!(invalid_json["status"], "error");
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn products_orders_and_payments_flow() {
    ensure_base_url();
    common::ensure_server_running().await;
    let client = Client::new();
    let token = common::register_and_login(&client).await;

    // Create supporting data
    let (category_uuid, _) = helpers::create_category(&client, &token).await;
    let product_resp = helpers::create_product(&client, &token, &category_uuid, None, 55.0).await;
    let product_uuid = product_resp["data"]["product"]["uuid"]
        .as_str()
        .expect("product uuid")
        .to_string();
    let product_name = product_resp["data"]["product"]["name"]
        .as_str()
        .expect("product name")
        .to_string();

    // Get product by id
    let product_get = client
        .get(format!(
            "{}/api/v1/products/{}",
            common::base_url(),
            product_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("get product");
    assert_eq!(
        product_get.status(),
        StatusCode::OK,
        "get product should be 200"
    );
    let product_get_json: Value = product_get.json().await.expect("product get json");
    assert_eq!(
        product_get_json["data"]["product"]["name"].as_str(),
        Some(product_name.as_str())
    );

    // Update product
    let updated_name = format!("{} Updated", product_name);
    let update_res = client
        .put(format!(
            "{}/api/v1/products/{}",
            common::base_url(),
            product_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "name": updated_name,
            "price": 60.0,
            "status": "INACTIVE"
        }))
        .send()
        .await
        .expect("update product");
    assert_eq!(
        update_res.status(),
        StatusCode::OK,
        "update product should be 200"
    );
    let update_json: Value = update_res.json().await.expect("update product json");
    assert_eq!(
        update_json["data"]["product"]["name"].as_str(),
        Some(updated_name.as_str())
    );
    assert_eq!(
        update_json["data"]["product"]["status"].as_str(),
        Some("INACTIVE")
    );

    // List products with search
    let list_res = client
        .get(format!(
            "{}/api/v1/products?search={}",
            common::base_url(),
            updated_name.split_whitespace().next().unwrap()
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("list products");
    assert_eq!(
        list_res.status(),
        StatusCode::OK,
        "list products should be 200"
    );
    let list_json: Value = list_res.json().await.expect("list products json");
    assert!(
        list_json["data"]["products"].is_array(),
        "products should be array"
    );

    // Create order referencing the product
    let order_resp = helpers::create_order(&client, &token, &product_uuid).await;
    let order_uuid = order_resp["data"]["uuid"]
        .as_str()
        .expect("order uuid")
        .to_string();

    // Get order by id
    let order_get = client
        .get(format!(
            "{}/api/v1/orders/{}",
            common::base_url(),
            order_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("get order");
    assert_eq!(
        order_get.status(),
        StatusCode::OK,
        "get order should be 200"
    );

    // Update order status
    let status_res = client
        .patch(format!(
            "{}/api/v1/orders/{}/status",
            common::base_url(),
            order_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({ "status": "PAID" }))
        .send()
        .await
        .expect("update order status");
    assert_eq!(
        status_res.status(),
        StatusCode::OK,
        "update order status should be 200"
    );

    // List orders
    let orders_list = client
        .get(format!("{}/api/v1/orders", common::base_url()))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("list orders");
    assert_eq!(
        orders_list.status(),
        StatusCode::OK,
        "list orders should be 200"
    );

    // Create payment for the order
    let payment_resp = helpers::create_payment(&client, &token, &order_uuid).await;
    let payment_uuid = payment_resp["data"]["uuid"]
        .as_str()
        .expect("payment uuid")
        .to_string();

    // Get payment by id
    let payment_get = client
        .get(format!(
            "{}/api/v1/payments/{}",
            common::base_url(),
            payment_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("get payment");
    assert_eq!(
        payment_get.status(),
        StatusCode::OK,
        "get payment should be 200"
    );

    // List payments filtered by order
    let payments_list = client
        .get(format!(
            "{}/api/v1/payments?order_uuid={}",
            common::base_url(),
            order_uuid
        ))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .expect("list payments");
    assert_eq!(
        payments_list.status(),
        StatusCode::OK,
        "list payments should be 200"
    );
    let payments_json: Value = payments_list.json().await.expect("payments list json");
    assert!(payments_json["data"]["payments"].is_array());
}
