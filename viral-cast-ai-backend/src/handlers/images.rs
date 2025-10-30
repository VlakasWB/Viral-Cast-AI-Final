use std::path::Path;
use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use tokio::fs;
use uuid::Uuid;

use crate::{dto::api::ApiResponse, AppState};

pub async fn upload_product_image_handler(
    State(_data): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    println!("🔥 Upload handler called!");
    tracing::info!("Starting image upload process");
    // Create uploads directory if it doesn't exist
    let upload_dir = "uploads/products";
    if let Err(_) = fs::create_dir_all(upload_dir).await {
        let error_response = json!({
            "status": "error",
            "message": "Failed to create upload directory"
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        println!("🔥 Error reading multipart field: {}", e);
        tracing::error!("Multipart field error: {}", e);
        let error_response = json!({
            "status": "error",
            "message": format!("Invalid multipart data: {}", e)
        });
        (StatusCode::BAD_REQUEST, Json(error_response))
    })? {
        let name = field.name().unwrap_or("").to_string();
        tracing::debug!("Processing field: {}", name);

        if name == "image" || name == "file" {
            let filename = field.file_name().unwrap_or("").to_string();

            println!("🔥 Processing file: {}", filename);

            if filename.is_empty() {
                let error_response = json!({
                    "status": "fail",
                    "message": "No file provided"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            // Validate file extension
            let allowed_extensions = ["jpg", "jpeg", "png", "gif", "webp"];
            let file_extension = Path::new(&filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !allowed_extensions.contains(&file_extension.as_str()) {
                let error_response = json!({
                    "status": "fail",
                    "message": "Invalid file type. Only JPG, JPEG, PNG, GIF, and WebP are allowed"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            // Generate unique filename
            let unique_filename = format!("{}.{}", Uuid::new_v4(), file_extension);
            let file_path = format!("{}/{}", upload_dir, unique_filename);

            // Get file data
            let data = match field.bytes().await {
                Ok(data) => {
                    println!("🔥 File data read successfully, size: {} bytes", data.len());
                    data
                }
                Err(e) => {
                    println!("🔥 Error reading file data: {}", e);
                    let error_response = json!({
                        "status": "error",
                        "message": format!("Failed to read file data: {}", e)
                    });
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
                }
            };

            // Validate file size (max 5MB)
            if data.len() > 5 * 1024 * 1024 {
                let error_response = json!({
                    "status": "fail",
                    "message": "File size too large. Maximum size is 5MB"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            // Save file
            match fs::write(&file_path, &data).await {
                Ok(_) => {
                    // Return the URL path
                    let image_url = format!("/uploads/products/{}", unique_filename);

                    let json_response = ApiResponse {
                        code: 200,
                        status: "success".to_string(),
                        message: "Image uploaded successfully".to_string(),
                        data: json!({
                            "image_url": image_url,
                            "filename": unique_filename,
                            "original_filename": filename,
                            "file_size": data.len()
                        }),
                        errors: json!(null),
                    };

                    return Ok((StatusCode::OK, Json(json_response)));
                }
                Err(_) => {
                    let error_response = json!({
                        "status": "error",
                        "message": "Failed to save file"
                    });
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
                }
            }
        }
    }

    let error_response = json!({
        "status": "fail",
        "message": "No image field found in the request (expected 'image' atau 'file')"
    });
    Err((StatusCode::BAD_REQUEST, Json(error_response)))
}

// Alternative handler with explicit content-type validation
pub async fn upload_product_image_handler_v2(
    State(_data): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    println!("🔥 Upload handler v2 called!");
    tracing::info!("Starting image upload process v2");

    // Validate content-type header
    if let Some(content_type) = headers.get("content-type") {
        let content_type_str = content_type.to_str().unwrap_or("");
        tracing::info!("Content-Type: {}", content_type_str);

        if !content_type_str.starts_with("multipart/form-data") {
            let error_response = json!({
                "status": "error",
                "message": "Content-Type must be multipart/form-data"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }

        // Check if boundary is present
        if !content_type_str.contains("boundary=") {
            let error_response = json!({
                "status": "error",
                "message": "Missing boundary in Content-Type header"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    } else {
        let error_response = json!({
            "status": "error",
            "message": "Missing Content-Type header"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Create uploads directory if it doesn't exist
    let upload_dir = "uploads/products";
    if let Err(_) = fs::create_dir_all(upload_dir).await {
        let error_response = json!({
            "status": "error",
            "message": "Failed to create upload directory"
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        println!("🔥 Error reading multipart field: {}", e);
        tracing::error!("Multipart field error: {}", e);
        let error_response = json!({
            "status": "error",
            "message": format!("Invalid multipart data: {}", e)
        });
        (StatusCode::BAD_REQUEST, Json(error_response))
    })? {
        let name = field.name().unwrap_or("").to_string();
        tracing::debug!("Processing field: {}", name);

        if name == "image" || name == "file" {
            let filename = field.file_name().unwrap_or("").to_string();

            println!("🔥 Processing file: {}", filename);

            if filename.is_empty() {
                let error_response = json!({
                    "status": "fail",
                    "message": "No file provided"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            // Validate file extension
            let allowed_extensions = ["jpg", "jpeg", "png", "gif", "webp"];
            let file_extension = Path::new(&filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !allowed_extensions.contains(&file_extension.as_str()) {
                let error_response = json!({
                    "status": "fail",
                    "message": "Invalid file type. Only JPG, JPEG, PNG, GIF, and WebP are allowed"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            // Generate unique filename
            let unique_filename = format!("{}.{}", Uuid::new_v4(), file_extension);
            let file_path = format!("{}/{}", upload_dir, unique_filename);

            // Get file data
            let data = match field.bytes().await {
                Ok(data) => {
                    println!("🔥 File data read successfully, size: {} bytes", data.len());
                    tracing::info!("File data read successfully, size: {} bytes", data.len());
                    data
                }
                Err(e) => {
                    println!("🔥 Error reading file data: {}", e);
                    tracing::error!("Error reading file data: {}", e);
                    let error_response = json!({
                        "status": "error",
                        "message": format!("Failed to read file data: {}", e)
                    });
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
                }
            };

            // Validate file size (max 5MB)
            if data.len() > 5 * 1024 * 1024 {
                let error_response = json!({
                    "status": "fail",
                    "message": "File size too large. Maximum size is 5MB"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            // Save file
            match fs::write(&file_path, &data).await {
                Ok(_) => {
                    // Return the URL path
                    let image_url = format!("/uploads/products/{}", unique_filename);

                    let json_response = ApiResponse {
                        code: 200,
                        status: "success".to_string(),
                        message: "Image uploaded successfully".to_string(),
                        data: json!({
                            "image_url": image_url,
                            "filename": unique_filename,
                            "original_filename": filename,
                            "file_size": data.len()
                        }),
                        errors: json!(null),
                    };

                    return Ok((StatusCode::OK, Json(json_response)));
                }
                Err(e) => {
                    tracing::error!("Failed to save file: {}", e);
                    let error_response = json!({
                        "status": "error",
                        "message": format!("Failed to save file: {}", e)
                    });
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
                }
            }
        }
    }

    let error_response = json!({
        "status": "fail",
        "message": "No image field found in the request (expected 'image' atau 'file')"
    });
    Err((StatusCode::BAD_REQUEST, Json(error_response)))
}

pub async fn delete_product_image_handler(
    State(_data): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let image_url = match body.get("image_url").and_then(|v| v.as_str()) {
        Some(url) => url,
        None => {
            let error_response = json!({
                "status": "fail",
                "message": "image_url is required"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Extract filename from URL
    let filename = match image_url.strip_prefix("/uploads/products/") {
        Some(name) => name,
        None => {
            let error_response = json!({
                "status": "fail",
                "message": "Invalid image URL format"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    let file_path = format!("uploads/products/{}", filename);

    // Check if file exists and delete it
    match fs::remove_file(&file_path).await {
        Ok(_) => {
            let json_response = ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "Image deleted successfully".to_string(),
                data: json!({
                    "deleted_image_url": image_url
                }),
                errors: json!(null),
            };

            Ok((StatusCode::OK, Json(json_response)))
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to delete image file or file not found"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

// Upload user profile image
pub async fn upload_user_image_handler(
    State(_data): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    // Validate content-type header upfront
    if let Some(content_type) = headers.get("content-type") {
        let content_type_str = content_type.to_str().unwrap_or("");
        if !content_type_str.starts_with("multipart/form-data") {
            let error_response = json!({
                "status": "error",
                "message": "Content-Type must be multipart/form-data"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
        if !content_type_str.contains("boundary=") {
            let error_response = json!({
                "status": "error",
                "message": "Missing boundary in Content-Type header"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    } else {
        let error_response = json!({
            "status": "error",
            "message": "Missing Content-Type header"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let upload_dir = "uploads/users";
    if let Err(_) = fs::create_dir_all(upload_dir).await {
        let error_response = json!({
            "status": "error",
            "message": "Failed to create upload directory"
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        let error_response = json!({
            "status": "error",
            "message": format!("Invalid multipart data: {}", e)
        });
        (StatusCode::BAD_REQUEST, Json(error_response))
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name == "image" || name == "file" {
            let filename = field.file_name().unwrap_or("").to_string();
            if filename.is_empty() {
                let error_response = json!({
                    "status": "fail",
                    "message": "No file provided"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            let allowed_extensions = ["jpg", "jpeg", "png", "gif", "webp"];
            let file_extension = Path::new(&filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !allowed_extensions.contains(&file_extension.as_str()) {
                let error_response = json!({
                    "status": "fail",
                    "message": "Invalid file type. Only JPG, JPEG, PNG, GIF, and WebP are allowed"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            let unique_filename = format!("{}.{}", Uuid::new_v4(), file_extension);
            let file_path = format!("{}/{}", upload_dir, unique_filename);

            let data = match field.bytes().await {
                Ok(data) => data,
                Err(e) => {
                    // Treat parsing errors on file read as BAD_REQUEST to guide client
                    let error_response = json!({
                        "status": "error",
                        "message": format!("Error parsing multipart/form-data request: {}", e)
                    });
                    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
                }
            };

            if data.len() > 5 * 1024 * 1024 {
                let error_response = json!({
                    "status": "fail",
                    "message": "File size too large. Maximum size is 5MB"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            match fs::write(&file_path, &data).await {
                Ok(_) => {
                    let image_url = format!("/uploads/users/{}", unique_filename);
                    let json_response = ApiResponse {
                        code: 200,
                        status: "success".to_string(),
                        message: "Image uploaded successfully".to_string(),
                        data: json!({
                            "image_url": image_url,
                            "filename": unique_filename,
                            "original_filename": filename,
                            "file_size": data.len()
                        }),
                        errors: json!(null),
                    };
                    return Ok((StatusCode::OK, Json(json_response)));
                }
                Err(e) => {
                    let error_response = json!({
                        "status": "error",
                        "message": format!("Failed to save file: {}", e)
                    });
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
                }
            }
        }
    }

    let error_response = json!({
        "status": "fail",
        "message": "No image field found in the request (expected 'image' or 'file')"
    });
    Err((StatusCode::BAD_REQUEST, Json(error_response)))
}

// Upload store brand image
pub async fn upload_store_brand_image_handler(
    State(_data): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let upload_dir = "uploads/stores";
    if let Err(_) = fs::create_dir_all(upload_dir).await {
        let error_response = json!({
            "status": "error",
            "message": "Failed to create upload directory"
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        let error_response = json!({
            "status": "error",
            "message": format!("Invalid multipart data: {}", e)
        });
        (StatusCode::BAD_REQUEST, Json(error_response))
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name == "image" || name == "file" {
            let filename = field.file_name().unwrap_or("").to_string();
            if filename.is_empty() {
                let error_response = json!({
                    "status": "fail",
                    "message": "No file provided"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            let allowed_extensions = ["jpg", "jpeg", "png", "gif", "webp"];
            let file_extension = Path::new(&filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !allowed_extensions.contains(&file_extension.as_str()) {
                let error_response = json!({
                    "status": "fail",
                    "message": "Invalid file type. Only JPG, JPEG, PNG, GIF, and WebP are allowed"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            let unique_filename = format!("{}.{}", Uuid::new_v4(), file_extension);
            let file_path = format!("{}/{}", upload_dir, unique_filename);

            let data = match field.bytes().await {
                Ok(data) => data,
                Err(e) => {
                    let error_response = json!({
                        "status": "error",
                        "message": format!("Failed to read file data: {}", e)
                    });
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
                }
            };

            if data.len() > 5 * 1024 * 1024 {
                let error_response = json!({
                    "status": "fail",
                    "message": "File size too large. Maximum size is 5MB"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            match fs::write(&file_path, &data).await {
                Ok(_) => {
                    let brand_url = format!("/uploads/stores/{}", unique_filename);
                    let json_response = ApiResponse {
                        code: 200,
                        status: "success".to_string(),
                        message: "Image uploaded successfully".to_string(),
                        data: json!({
                            "brand_url": brand_url,
                            "filename": unique_filename,
                            "original_filename": filename,
                            "file_size": data.len()
                        }),
                        errors: json!(null),
                    };
                    return Ok((StatusCode::OK, Json(json_response)));
                }
                Err(_) => {
                    let error_response = json!({
                        "status": "error",
                        "message": "Failed to save file"
                    });
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
                }
            }
        }
    }

    let error_response = json!({
        "status": "fail",
        "message": "No image field found in the request (expected 'image' atau 'file')"
    });
    Err((StatusCode::BAD_REQUEST, Json(error_response)))
}
