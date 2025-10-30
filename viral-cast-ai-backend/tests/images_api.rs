#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use reqwest::multipart::{Form, Part};
use reqwest::{Client, StatusCode};

mod common;

fn ensure_base_url() {
    std::env::set_var("BASE_URL", "http://localhost:12000");
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn upload_user_image_success_and_bad_content_type() {
    ensure_base_url();
    common::ensure_server_running().await;

    let client = Client::new();
    let token = common::register_and_login(&client).await;

    // Successful multipart upload
    let upload_url = format!("{}/api/v1/images/upload/user", common::base_url());
    let img_path = {
        let mut p = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set"),
        );
        p.push("pexels-edwardeyer-1049620.jpg");
        p
    };
    assert!(
        img_path.exists(),
        "sample image should exist at {:?}",
        img_path
    );

    let file_bytes = tokio::fs::read(&img_path).await.expect("read sample image");
    let file_name = img_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("sample.jpg")
        .to_string();
    let part = Part::bytes(file_bytes)
        .file_name(file_name)
        .mime_str("image/jpeg")
        .expect("set mime");
    let form = Form::new().part("image", part);

    let res = client
        .post(upload_url.clone())
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .await
        .expect("multipart upload resp");
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "upload user image should be 200"
    );
    let json: serde_json::Value = res.json().await.expect("upload json");
    assert_eq!(json["code"], 200);
    assert_eq!(json["status"], "success");
    let url = json["data"]["image_url"].as_str().expect("image_url");
    assert!(
        url.starts_with("/uploads/users/"),
        "image_url should point to users uploads"
    );
    let size = json["data"]["file_size"].as_u64().expect("file_size");
    assert!(size > 0, "file_size should be > 0");

    // Bad Content-Type (non-multipart) should yield 400
    let bad_res = client
        .post(upload_url.clone())
        .bearer_auth(&token)
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .expect("bad content-type resp");
    assert_eq!(
        bad_res.status(),
        StatusCode::BAD_REQUEST,
        "non-multipart upload should be 400"
    );
}
