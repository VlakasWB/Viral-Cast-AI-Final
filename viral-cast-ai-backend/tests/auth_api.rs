#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, StatusCode,
};
use serde_json::json;
use std::process::Command;
use uuid::Uuid;

fn base_url() -> String {
    std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:4000".to_string())
}

fn extract_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let prefix = format!("{}=", name);
    headers
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|cookie| {
            if let Some(idx) = cookie.find(&prefix) {
                let rest = &cookie[idx + prefix.len()..];
                let end = rest.find(';').unwrap_or(rest.len());
                Some(rest[..end].to_string())
            } else {
                None
            }
        })
}

fn cookie_header(refresh_token: &str, logged_in: Option<bool>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let cookie_val = if let Some(li) = logged_in {
        format!(
            "refresh_token={}; logged_in={}",
            refresh_token,
            if li { "true" } else { "false" }
        )
    } else {
        format!("refresh_token={}", refresh_token)
    };
    headers.insert(
        reqwest::header::COOKIE,
        HeaderValue::from_str(&cookie_val).expect("valid cookie header"),
    );
    headers
}

async fn ensure_server_running() {
    let client = Client::new();
    let health_url = format!("{}/api/v1/healthchecker", base_url());

    // If already up, return
    if client
        .get(health_url.clone())
        .send()
        .await
        .map(|r| r.status() == StatusCode::OK)
        .unwrap_or(false)
    {
        return;
    }

    // Spawn compiled binary directly (avoids Cargo locking issues)
    let exe_path = {
        let mut p = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set"),
        );
        p.push("target");
        p.push("debug");
        p.push(format!(
            "viral_cast_ai_backend{}",
            std::env::consts::EXE_SUFFIX
        ));
        p
    };

    let _child = Command::new(&exe_path)
        .spawn()
        .expect("spawn server binary");

    // Wait until healthy
    for _ in 0..50 {
        if let Ok(resp) = client.get(health_url.clone()).send().await {
            if resp.status() == StatusCode::OK {
                return;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    panic!("Server did not become ready at {}", health_url);
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn healthchecker_ok() {
    ensure_server_running().await;
    let client = Client::new();
    let url = format!("{}/api/v1/healthchecker", base_url());
    let res = client
        .get(url)
        .send()
        .await
        .expect("healthchecker reachable");
    assert_eq!(res.status(), StatusCode::OK);
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn auth_flow_register_login_refresh_me_logout() {
    ensure_server_running().await;
    let client = Client::builder().build().expect("client");

    // Unique test user
    let uname = format!("testuser_{}", &Uuid::new_v4().to_string()[..8]);
    let email = format!("{}@example.com", uname);
    let password = "Passw0rd!";

    // Register
    let reg_url = format!("{}/api/v1/auth/register", base_url());
    let reg_body = json!({
        "username": uname,
        "email": email,
        "password": password,
        "name": "Auth Tester",
        "phone": "08123456789"
    });
    let reg_res = client
        .post(reg_url)
        .json(&reg_body)
        .send()
        .await
        .expect("register response");
    assert_eq!(
        reg_res.status(),
        StatusCode::CREATED,
        "register should be 201"
    );

    let reg_headers = reg_res.headers().clone();
    let reg_json: serde_json::Value = reg_res.json().await.expect("register json");
    let access_token_reg = reg_json["data"]["access_token"]
        .as_str()
        .expect("reg access token");
    let refresh_cookie =
        extract_cookie_value(&reg_headers, "refresh_token").expect("refresh cookie");

    // Refresh access token using cookie
    let refresh_url = format!("{}/api/v1/auth/refresh", base_url());
    let res_refresh = client
        .get(refresh_url)
        .headers(cookie_header(&refresh_cookie, Some(true)))
        .send()
        .await
        .expect("refresh response");
    assert_eq!(
        res_refresh.status(),
        StatusCode::OK,
        "refresh should be 200"
    );
    let refresh_json: serde_json::Value = res_refresh.json().await.expect("refresh json");
    let access_token_new = refresh_json["data"]["access_token"]
        .as_str()
        .expect("new access token");

    // Get me using latest access token
    let me_url = format!("{}/api/v1/users/me", base_url());
    let me_res = client
        .get(me_url.clone())
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token_new),
        )
        .send()
        .await
        .expect("me response");
    assert_eq!(me_res.status(), StatusCode::OK, "me should be 200");
    let me_json: serde_json::Value = me_res.json().await.expect("me json");
    assert_eq!(me_json["data"]["username"].as_str(), Some(uname.as_str()));

    // Logout (requires refresh token cookie and typically Authorization)
    let logout_url = format!("{}/api/v1/auth/logout", base_url());
    let logout_res = client
        .post(logout_url)
        .headers(cookie_header(&refresh_cookie, Some(true)))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token_new),
        )
        .send()
        .await
        .expect("logout response");
    assert_eq!(logout_res.status(), StatusCode::OK, "logout should be 200");

    // Me should now be unauthorized with old token
    let me_res2 = client
        .get(me_url)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token_new),
        )
        .send()
        .await
        .expect("me response after logout");
    assert_eq!(
        me_res2.status(),
        StatusCode::UNAUTHORIZED,
        "me after logout should be 401"
    );

    // Login again
    let login_url = format!("{}/api/v1/auth/login", base_url());
    let login_body = json!({"username": uname, "password": password});
    let login_res = client
        .post(login_url)
        .json(&login_body)
        .send()
        .await
        .expect("login response");
    assert_eq!(login_res.status(), StatusCode::OK, "login should be 200");
    let login_headers = login_res.headers().clone();
    let login_json: serde_json::Value = login_res.json().await.expect("login json");
    let access_token_login = login_json["data"]["access_token"]
        .as_str()
        .expect("login access token");
    let refresh_cookie2 =
        extract_cookie_value(&login_headers, "refresh_token").expect("refresh cookie 2");

    // Me should be ok again
    let me_res3 = Client::new()
        .get(format!("{}/api/v1/users/me", base_url()))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token_login),
        )
        .send()
        .await
        .expect("me response after login");
    assert_eq!(
        me_res3.status(),
        StatusCode::OK,
        "me after login should be 200"
    );

    // Refresh with the new cookie should work
    let res_refresh2 = Client::new()
        .get(format!("{}/api/v1/auth/refresh", base_url()))
        .headers(cookie_header(&refresh_cookie2, Some(true)))
        .send()
        .await
        .expect("refresh response 2");
    assert_eq!(
        res_refresh2.status(),
        StatusCode::OK,
        "refresh2 should be 200"
    );
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn login_wrong_password() {
    ensure_server_running().await;
    let client = Client::new();
    let uname = format!("testuser_{}", &Uuid::new_v4().to_string()[..8]);
    let email = format!("{}@example.com", uname);
    let password = "Passw0rd!";

    // Register first
    let reg_url = format!("{}/api/v1/auth/register", base_url());
    let reg_body = json!({
        "username": uname,
        "email": email,
        "password": password,
        "name": "Auth Tester",
        "phone": "08123456789"
    });
    let reg_res = client
        .post(reg_url)
        .json(&reg_body)
        .send()
        .await
        .expect("register");
    assert_eq!(reg_res.status(), StatusCode::CREATED);

    // Try wrong password
    let login_url = format!("{}/api/v1/auth/login", base_url());
    let login_body = json!({"username": uname, "password": "WRONG-PASS"});
    let res = client
        .post(login_url)
        .json(&login_body)
        .send()
        .await
        .expect("login wrong");
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn refresh_without_cookie_forbidden() {
    ensure_server_running().await;
    let client = Client::new();
    let refresh_url = format!("{}/api/v1/auth/refresh", base_url());
    let res = client.get(refresh_url).send().await.expect("refresh");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn register_duplicate_conflict() {
    ensure_server_running().await;
    let client = Client::new();
    let uname = format!("testuser_{}", &Uuid::new_v4().to_string()[..8]);
    let email = format!("{}@example.com", uname);
    let password = "Passw0rd!";

    let reg_url = format!("{}/api/v1/auth/register", base_url());
    let reg_body = json!({
        "username": uname,
        "email": email,
        "password": password,
        "name": "Auth Tester",
        "phone": "08123456789"
    });

    let res1 = client
        .post(&reg_url)
        .json(&reg_body)
        .send()
        .await
        .expect("register1");
    assert_eq!(res1.status(), StatusCode::CREATED);

    let res2 = client
        .post(&reg_url)
        .json(&reg_body)
        .send()
        .await
        .expect("register2");
    assert_eq!(res2.status(), StatusCode::CONFLICT);
}
