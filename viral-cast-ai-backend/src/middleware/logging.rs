use axum::{extract::Request, http::header::USER_AGENT, middleware::Next, response::Response};
use chrono::Utc;
use serde_json::json;
use std::{path::PathBuf, time::Instant};
use tokio::{
    fs::{create_dir_all, OpenOptions},
    io::AsyncWriteExt,
};
use tracing::warn;

const BASE_LOG_DIR: &str = "logs";
const API_LOG_DIR: &str = "api";
const LOG_FILE_NAME: &str = "requests.log";
const SKIP_SEGMENTS: &[&str] = &["", "api", "v1", "v1_1", "v2"];
const DEFAULT_SEGMENT: &str = "misc";

fn alias_segment(segment: &str) -> String {
    match segment {
        "units-of-measure" => "uoms".to_string(),
        "ingredient-catalog" => "ingredients".to_string(),
        "ingredient-stock-moves" => "ingredient-stock-moves".to_string(),
        "ingredient-stocks" => "ingredient-stocks".to_string(),
        "ingredient-market-prices" => "ingredient-market-prices".to_string(),
        "ai" => "ai".to_string(),
        "rag" => "ai-rag".to_string(),
        "stores" => "stores".to_string(),
        "profiles" => "profiles".to_string(),
        "orders" => "orders".to_string(),
        "payments" => "payments".to_string(),
        "products" => "products".to_string(),
        "categories" => "categories".to_string(),
        "roles" => "roles".to_string(),
        "images" => "images".to_string(),
        "forecast-daily" => "forecast-daily".to_string(),
        "weather" | "weather-bmkg" => "weather-bmkg".to_string(),
        other => other.to_string(),
    }
}

pub async fn api_logger(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();
    let start = Instant::now();

    let response = next.run(req).await;

    let elapsed_ms = start.elapsed().as_millis();
    let status = response.status();
    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string());

    let log_entry = json!({
        "timestamp": Utc::now().to_rfc3339(),
        "method": method.as_str(),
        "path": uri.path(),
        "query": uri.query(),
        "status": status.as_u16(),
        "elapsed_ms": elapsed_ms,
        "user_agent": user_agent,
    });

    if let Err(error) = write_log_entry(uri.path(), &log_entry).await {
        warn!(%error, "Failed to write API request log");
    }

    response
}

async fn write_log_entry(path: &str, entry: &serde_json::Value) -> std::io::Result<()> {
    let mut dir_path = PathBuf::from(BASE_LOG_DIR);
    dir_path.push(API_LOG_DIR);

    let segments = determine_segments(path);

    if segments.is_empty() {
        dir_path.push("root");
    } else {
        for segment in &segments {
            dir_path.push(segment);
        }
    }

    create_dir_all(&dir_path).await?;

    let file_path = dir_path.join(LOG_FILE_NAME);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .await?;

    let mut log_line = serde_json::to_vec(entry).unwrap_or_default();
    log_line.push(b'\n');
    file.write_all(&log_line).await
}

fn sanitize_segment(segment: &str) -> String {
    segment
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn determine_segments(path: &str) -> Vec<String> {
    let mut collected = Vec::new();

    for raw in path.trim_matches('/').split('/') {
        let lower = raw.to_lowercase();
        if SKIP_SEGMENTS.contains(&lower.as_str()) {
            continue;
        }

        let segment = if is_uuid_like(&lower) {
            "uuid".to_string()
        } else if lower.chars().all(|c| c.is_ascii_digit()) {
            "id".to_string()
        } else {
            sanitize_segment(&alias_segment(&lower))
        };

        if segment.is_empty() {
            continue;
        }

        collected.push(segment);
        if collected.len() >= 2 {
            break;
        }
    }

    if collected.is_empty() {
        collected.push(DEFAULT_SEGMENT.to_string());
    }

    collected
}

fn is_uuid_like(value: &str) -> bool {
    value.len() == 36 && value.chars().filter(|&c| c == '-').count() == 4
}
