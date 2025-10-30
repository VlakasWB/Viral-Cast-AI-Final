#![cfg_attr(
    not(feature = "integration-tests"),
    allow(dead_code, unused_imports, unused_variables)
)]
use reqwest::{
    multipart::{Form, Part},
    Client,
};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;

mod common;
use common::{base_url, ensure_server_running, register_and_login};

#[derive(Clone, Deserialize)]
struct PostmanCollection {
    item: Vec<Item>,
}

#[derive(Clone, Deserialize)]
struct Item {
    name: Option<String>,
    item: Option<Vec<Item>>, // nested items
    request: Option<Request>,
}

#[derive(Clone, Deserialize)]
struct Request {
    method: String,
    header: Option<Vec<Header>>, // optional headers
    body: Option<Body>,          // optional body
    url: Url,
}

#[derive(Clone, Deserialize)]
struct Header {
    key: String,
    value: Option<String>,
}

#[derive(Clone, Deserialize)]
struct Body {
    mode: Option<String>,
    raw: Option<String>,
    formdata: Option<Vec<FormDataEntry>>,
    // urlencoded not used in current collection
}

#[derive(Clone, Deserialize)]
struct FormDataEntry {
    key: String,
    #[serde(rename = "type")]
    type_: Option<String>,
    value: Option<String>,
    src: Option<serde_json::Value>,
}

#[derive(Clone, Deserialize)]
struct Url {
    raw: Option<String>,
}

fn gather_requests(items: &[Item], out: &mut Vec<(String, Request)>) {
    for it in items {
        if let Some(req) = &it.request {
            let name = it.name.clone().unwrap_or_else(|| "".to_string());
            out.push((name, req.clone()));
        }
        if let Some(children) = &it.item {
            gather_requests(children, out);
        }
    }
}

fn substitute_vars(text: &str, base: &str, token: &str) -> String {
    text.replace("{{base_url}}", base)
        .replace("{{auth_token}}", token)
}

#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires external services"
)]
#[tokio::test]
async fn run_all_requests_in_postman_collection() {
    ensure_server_running().await;
    let client = Client::new();

    // fresh token for requests that require auth
    let access_token = register_and_login(&client).await;
    let base = base_url();

    // Load Postman collection file
    let collection_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("postman_collections")
        .join("VIRAL CAST AI.postman_collection copy.json");
    let json_text =
        fs::read_to_string(&collection_path).expect("failed to read postman collection file");
    let collection: PostmanCollection =
        serde_json::from_str(&json_text).expect("failed to parse postman collection json");

    // Flatten items into (name, request)
    let mut all: Vec<(String, Request)> = Vec::new();
    gather_requests(&collection.item, &mut all);

    // Deduplicate by METHOD + URL RAW
    let mut seen: HashSet<String> = HashSet::new();
    let mut unique: Vec<(String, Request)> = Vec::new();
    for (name, req) in all.into_iter() {
        let raw = req.url.raw.clone().unwrap_or_else(|| "".to_string());
        let key = format!("{} {}", req.method.to_uppercase(), raw);
        if seen.insert(key) {
            unique.push((name, req));
        }
    }

    assert!(
        !unique.is_empty(),
        "No requests found in Postman collection"
    );

    // Run each unique request
    for (name, req) in unique.into_iter() {
        // Build URL from raw with substitutions
        let mut url = substitute_vars(
            &req.url.raw.clone().unwrap_or_default(),
            &base,
            &access_token,
        );

        // Some items might not include base_url, patch if missing
        if url.starts_with("/") {
            url = format!("{}{}", base, url);
        }
        if !url.starts_with("http") {
            // last resort, assume it's relative to base
            url = format!(
                "{}/{}",
                base.trim_end_matches('/'),
                url.trim_start_matches('/')
            );
        }

        // Initialize request builder according to method
        let mut rb = match req.method.to_uppercase().as_str() {
            "GET" => client.get(url.clone()),
            "POST" => client.post(url.clone()),
            "PUT" => client.put(url.clone()),
            "PATCH" => client.patch(url.clone()),
            "DELETE" => client.delete(url.clone()),
            _ => {
                println!("[SKIP] Unsupported method: {} for {}", req.method, url);
                continue;
            }
        };

        // Apply headers with variable substitution
        let mut has_auth_header = false;
        if let Some(headers) = &req.header {
            for h in headers {
                let val =
                    substitute_vars(&h.value.clone().unwrap_or_default(), &base, &access_token);
                if h.key.eq_ignore_ascii_case("authorization") {
                    has_auth_header = true;
                }
                rb = rb.header(&h.key, val);
            }
        }

        // If not explicitly set but probably needs auth (heuristic)
        if !has_auth_header && url.contains("/api/") && !url.contains("/auth/") {
            rb = rb.bearer_auth(&access_token);
        }

        // Handle body: raw JSON or multipart form-data
        if let Some(body) = &req.body {
            match body.mode.as_deref() {
                Some("raw") => {
                    if let Some(raw) = &body.raw {
                        // Try to parse as JSON, fallback to text
                        match serde_json::from_str::<serde_json::Value>(raw) {
                            Ok(json_val) => {
                                rb = rb.json(&json_val);
                            }
                            Err(_) => {
                                rb = rb.body(raw.clone());
                            }
                        }
                    }
                }
                Some("formdata") => {
                    let mut form = Form::new();
                    if let Some(entries) = &body.formdata {
                        for e in entries {
                            let t = e.type_.as_deref().unwrap_or("text");
                            if t == "file" {
                                let file_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                                    .join("pexels-edwardeyer-1049620.jpg");
                                if file_path.exists() {
                                    let bytes = tokio::fs::read(&file_path)
                                        .await
                                        .expect("read attachment sample");
                                    let file_name = file_path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("sample.bin")
                                        .to_string();
                                    let part = Part::bytes(bytes)
                                        .file_name(file_name)
                                        .mime_str("application/octet-stream")
                                        .expect("set mime");
                                    form = form.part(e.key.clone(), part);
                                } else {
                                    form = form.text(e.key.clone(), "FILE_PLACEHOLDER");
                                }
                            } else {
                                form =
                                    form.text(e.key.clone(), e.value.clone().unwrap_or_default());
                            }
                        }
                    }
                    rb = rb.multipart(form);
                }
                _ => {}
            }
        }

        println!("[RUN] {} -> {} {}", name, req.method, url);
        let resp = rb.send().await.expect("request send error");
        // Allow 2xx, 3xx, 4xx (client errors for invalid samples), but fail on 5xx
        let status = resp.status();
        assert!(
            !status.is_server_error(),
            "Server error for [{}] {} {} => {}",
            name,
            req.method,
            url,
            status
        );

        // Small delay to avoid overwhelming server
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}
