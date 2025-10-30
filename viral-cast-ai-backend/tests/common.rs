#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use reqwest::{Client, StatusCode};
use serde_json::json;
use std::process::Command;
use uuid::Uuid;

pub fn base_url() -> String {
    std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:4000".to_string())
}

pub async fn ensure_server_running() {
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

    // Copy compiled binary to a unique tmp path and spawn to avoid locking on Windows
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set");
    let src_exe = {
        let mut p = std::path::PathBuf::from(&manifest_dir);
        p.push("target");
        p.push("debug");
        p.push(format!(
            "viral_cast_ai_backend{}",
            std::env::consts::EXE_SUFFIX
        ));
        p
    };
    let tmp_dir = {
        let mut p = std::path::PathBuf::from(&manifest_dir);
        p.push("tmp");
        p.push("target-tests");
        p
    };
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir for copied exe");
    let runner_name = format!(
        "viral_cast_ai_backend_runner_{}{}",
        &Uuid::new_v4().to_string()[..8],
        std::env::consts::EXE_SUFFIX
    );
    let dest_exe = tmp_dir.join(runner_name);

    // Copy freshly built exe to tmp location and spawn from there
    std::fs::copy(&src_exe, &dest_exe).expect("copy server exe to tmp");
    let _child = Command::new(&dest_exe)
        .spawn()
        .expect("spawn server binary from tmp");

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

/// Registers a unique user and logs in, returning a fresh access token
pub async fn register_and_login(client: &Client) -> String {
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
        "name": "Integration Tester",
        "phone": "08123456789"
    });
    let reg_res = client
        .post(reg_url)
        .json(&reg_body)
        .send()
        .await
        .expect("register response");
    assert!(
        reg_res.status().is_success(),
        "register expected success, got {}",
        reg_res.status()
    );

    // Login
    let login_url = format!("{}/api/v1/auth/login", base_url());
    let login_body = json!({"username": uname, "password": password});
    let login_res = client
        .post(login_url)
        .json(&login_body)
        .send()
        .await
        .expect("login response");
    assert_eq!(login_res.status(), StatusCode::OK, "login should be 200");
    let login_json: serde_json::Value = login_res.json().await.expect("login json");
    let access_token = login_json["data"]["access_token"]
        .as_str()
        .expect("access token")
        .to_string();

    access_token
}
