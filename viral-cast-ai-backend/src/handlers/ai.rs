use axum::{extract::State, http::StatusCode, response::Json};
use chrono::{Timelike, Utc};
use regex;
use reqwest::Client;
use std::sync::Arc;
use uuid::Uuid;

use crate::repository::ai as ai_repository;
use crate::{
    dto::{
        ai::{
            AiConfigResponse, DetailedTokenUsageResponse, GroqApiRequest, GroqApiResponse,
            GroqChatRequest, GroqChatResponse, GroqChatUnlimitedRequest, GroqMessage,
            TokenMonitoringAlert, TokenUsageHistoryResponse, TokenUsageResponse,
            UpdateAiConfigRequest, UserInputControlRequest, UserInputControlResponse,
        },
        api::ApiResponse,
    },
    models::ai_config::{AiConfig, AiRequestLog, TokenUsage, UserInputControl},
    AppState,
};

// GROQ configuration will be read from environment variables
fn get_groq_api_key() -> String {
    std::env::var("GROQ_API_KEY").unwrap_or_else(|_| panic!("GROQ_API_KEY must be set"))
}

fn get_groq_api_url() -> String {
    std::env::var("GROQ_API_URL")
        .unwrap_or_else(|_| "https://api.groq.com/openai/v1/chat/completions".to_string())
}

fn get_groq_model() -> String {
    std::env::var("GROQ_MODEL").unwrap_or_else(|_| "llama-3.1-8b-instant".to_string())
}

pub async fn chat_with_ai(
    State(data): State<Arc<AppState>>,
    Json(body): Json<GroqChatRequest>,
) -> Result<Json<ApiResponse<GroqChatResponse>>, StatusCode> {
    // Extract user IP (in real implementation, this would come from request headers)
    let user_ip = "127.0.0.1"; // Placeholder - should be extracted from request

    // Check rate limiting
    match check_rate_limit(&data, user_ip).await {
        Ok(allowed) => {
            if !allowed {
                let error_response = GroqChatResponse {
                    response: "".to_string(),
                    tokens_used: 0,
                    tokens_remaining: None,
                    model: get_groq_model(),
                    success: false,
                    message: Some("Rate limit exceeded. Please try again later.".to_string()),
                };
                return Ok(Json(ApiResponse {
                    code: 429,
                    status: "error".to_string(),
                    message: "Rate limit exceeded".to_string(),
                    data: error_response,
                    errors: serde_json::json!({}),
                }));
            }
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    // Validate user input with new validation system
    if let Err(validation_error) = validate_user_input(&data, &body.prompt).await {
        let error_response = GroqChatResponse {
            response: "".to_string(),
            tokens_used: 0,
            tokens_remaining: None,
            model: get_groq_model(),
            success: false,
            message: Some(validation_error.clone()),
        };
        return Ok(Json(ApiResponse {
            code: 400,
            status: "error".to_string(),
            message: validation_error,
            data: error_response,
            errors: serde_json::json!({}),
        }));
    }

    // Get AI configuration
    let config = get_ai_config(&data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check token limits if enabled
    if config.token_limit_enabled {
        if let Err(limit_error) = check_token_limits(&data, &config).await {
            let error_response = GroqChatResponse {
                response: "".to_string(),
                tokens_used: 0,
                tokens_remaining: None,
                model: get_groq_model(),
                success: false,
                message: Some(limit_error.clone()),
            };
            return Ok(Json(ApiResponse {
                code: 429,
                status: "error".to_string(),
                message: limit_error,
                data: error_response,
                errors: serde_json::json!({}),
            }));
        }
    }

    // Increment rate limit counter
    if let Err(_) = increment_rate_limit(&data, user_ip).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Make request to Groq API
    match make_groq_request(&body).await {
        Ok(groq_response) => {
            // Log the request
            let _ = log_ai_request(&data, &body, &groq_response, true, None).await;

            // Update token usage
            let _ = update_token_usage(&data, groq_response.usage.total_tokens as i32).await;

            let response = GroqChatResponse {
                response: groq_response.choices[0].message.content.clone(),
                tokens_used: groq_response.usage.total_tokens,
                tokens_remaining: calculate_remaining_tokens(&data, &config).await,
                model: groq_response.model,
                success: true,
                message: None,
            };

            Ok(Json(ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "AI response generated successfully".to_string(),
                data: response,
                errors: serde_json::json!({}),
            }))
        }
        Err(error) => {
            // Log the failed request
            let _ = log_ai_request(
                &data,
                &body,
                &GroqApiResponse {
                    id: "".to_string(),
                    object: "".to_string(),
                    created: 0,
                    model: get_groq_model(),
                    choices: vec![],
                    usage: crate::dto::ai::GroqUsage {
                        prompt_tokens: 0,
                        completion_tokens: 0,
                        total_tokens: 0,
                    },
                },
                false,
                Some(error.clone()),
            )
            .await;

            let error_response = GroqChatResponse {
                response: "".to_string(),
                tokens_used: 0,
                tokens_remaining: None,
                model: get_groq_model(),
                success: false,
                message: Some(error.to_string()),
            };
            Ok(Json(ApiResponse {
                code: 500,
                status: "error".to_string(),
                message: format!("Failed to get AI response: {}", error),
                data: error_response,
                errors: serde_json::json!({}),
            }))
        }
    }
}

pub async fn chat_with_ai_unlimited(
    State(data): State<Arc<AppState>>,
    Json(body): Json<GroqChatUnlimitedRequest>,
) -> Result<Json<ApiResponse<GroqChatResponse>>, StatusCode> {
    // Extract user IP (in real implementation, this would come from request headers)
    let user_ip = "127.0.0.1"; // Placeholder - should be extracted from request

    // Hanya periksa rate limiting jika bypass_validation = false
    if !body.bypass_validation {
        match check_rate_limit(&data, user_ip).await {
            Ok(allowed) => {
                if !allowed {
                    let error_response = GroqChatResponse {
                        response: "".to_string(),
                        tokens_used: 0,
                        tokens_remaining: None,
                        model: get_groq_model(),
                        success: false,
                        message: Some("Rate limit exceeded. Please try again later.".to_string()),
                    };
                    return Ok(Json(ApiResponse {
                        code: 429,
                        status: "error".to_string(),
                        message: "Rate limit exceeded".to_string(),
                        data: error_response,
                        errors: serde_json::json!({}),
                    }));
                }
            }
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }

        // Validasi input pengguna hanya jika bypass_validation = false
        if let Err(validation_error) = validate_user_input(&data, &body.prompt).await {
            let error_response = GroqChatResponse {
                response: "".to_string(),
                tokens_used: 0,
                tokens_remaining: None,
                model: get_groq_model(),
                success: false,
                message: Some(validation_error.clone()),
            };
            return Ok(Json(ApiResponse {
                code: 400,
                status: "error".to_string(),
                message: validation_error,
                data: error_response,
                errors: serde_json::json!({}),
            }));
        }
    }

    // Get AI configuration
    let config = get_ai_config(&data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Periksa batas token hanya jika bypass_validation = false
    if !body.bypass_validation && config.token_limit_enabled {
        if let Err(limit_error) = check_token_limits(&data, &config).await {
            let error_response = GroqChatResponse {
                response: "".to_string(),
                tokens_used: 0,
                tokens_remaining: None,
                model: get_groq_model(),
                success: false,
                message: Some(limit_error.clone()),
            };
            return Ok(Json(ApiResponse {
                code: 429,
                status: "error".to_string(),
                message: limit_error,
                data: error_response,
                errors: serde_json::json!({}),
            }));
        }
    }

    // Increment rate limit counter hanya jika bypass_validation = false
    if !body.bypass_validation {
        if let Err(_) = increment_rate_limit(&data, user_ip).await {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Konversi ke GroqChatRequest untuk menggunakan fungsi make_groq_request yang sudah ada
    let request = GroqChatRequest {
        prompt: body.prompt.clone(),
        max_tokens: body.max_tokens,
        temperature: body.temperature,
    };

    // Make request to Groq API
    match make_groq_request(&request).await {
        Ok(groq_response) => {
            // Log the request
            let _ = log_ai_request(&data, &request, &groq_response, true, None).await;

            // Update token usage hanya jika bypass_validation = false
            if !body.bypass_validation {
                let _ = update_token_usage(&data, groq_response.usage.total_tokens as i32).await;
            }

            let response = GroqChatResponse {
                response: groq_response.choices[0].message.content.clone(),
                tokens_used: groq_response.usage.total_tokens,
                tokens_remaining: if body.bypass_validation {
                    None
                } else {
                    calculate_remaining_tokens(&data, &config).await
                },
                model: groq_response.model,
                success: true,
                message: None,
            };

            Ok(Json(ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "AI response generated successfully".to_string(),
                data: response,
                errors: serde_json::json!({}),
            }))
        }
        Err(error) => {
            // Log the failed request
            let _ = log_ai_request(
                &data,
                &request,
                &GroqApiResponse {
                    id: "".to_string(),
                    object: "".to_string(),
                    created: 0,
                    model: get_groq_model(),
                    choices: vec![],
                    usage: crate::dto::ai::GroqUsage {
                        prompt_tokens: 0,
                        completion_tokens: 0,
                        total_tokens: 0,
                    },
                },
                false,
                Some(error.clone()),
            )
            .await;

            let error_response = GroqChatResponse {
                response: "".to_string(),
                tokens_used: 0,
                tokens_remaining: None,
                model: get_groq_model(),
                success: false,
                message: Some(error.to_string()),
            };
            Ok(Json(ApiResponse {
                code: 500,
                status: "error".to_string(),
                message: format!("Failed to get AI response: {}", error),
                data: error_response,
                errors: serde_json::json!({}),
            }))
        }
    }
}

pub async fn get_ai_configuration(
    State(data): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<AiConfigResponse>>, StatusCode> {
    match get_ai_config(&data).await {
        Ok(config) => {
            let response = AiConfigResponse {
                input_validation_enabled: config.input_validation_enabled,
                token_limit_enabled: config.token_limit_enabled,
                max_input_length: config.max_input_length as u32,
                allowed_topics: config.allowed_topics,
            };

            Ok(Json(ApiResponse {
                code: 200,
                status: "success".to_string(),
                message: "AI configuration retrieved successfully".to_string(),
                data: response,
                errors: serde_json::json!({}),
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_ai_configuration(
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateAiConfigRequest>,
) -> Result<Json<ApiResponse<AiConfigResponse>>, StatusCode> {
    let mut config = get_ai_config(&data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(input_validation) = body.input_validation_enabled {
        config.input_validation_enabled = input_validation;
    }
    if let Some(token_limit) = body.token_limit_enabled {
        config.token_limit_enabled = token_limit;
    }
    if let Some(max_length) = body.max_input_length {
        config.max_input_length = max_length as i32;
    }

    ai_repository::update_ai_config(&data.db, &config)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = AiConfigResponse {
        input_validation_enabled: config.input_validation_enabled,
        token_limit_enabled: config.token_limit_enabled,
        max_input_length: config.max_input_length as u32,
        allowed_topics: config.allowed_topics,
    };

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "AI configuration updated successfully".to_string(),
        data: response,
        errors: serde_json::json!({}),
    }))
}

pub async fn get_token_usage(
    State(data): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<TokenUsageResponse>>, StatusCode> {
    let today = chrono::Utc::now().date_naive();
    let usage = ai_repository::get_token_usage_by_date(&data.db, today)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let config = get_ai_config(&data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tokens_used_today = usage.map(|u| u.tokens_used).unwrap_or(0) as u32;
    let daily_limit = config.daily_token_limit.map(|l| l as u32);
    let tokens_remaining = daily_limit.map(|limit| {
        if tokens_used_today >= limit {
            0
        } else {
            limit - tokens_used_today
        }
    });
    let response = TokenUsageResponse {
        total_tokens_used_today: tokens_used_today,
        tokens_remaining,
        daily_limit,
        last_reset: today.to_string(),
    };
    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Token usage retrieved successfully".to_string(),
        data: response,
        errors: serde_json::json!({}),
    }))
}

// Helper functions
async fn get_ai_config(data: &Arc<AppState>) -> Result<AiConfig, sqlx::Error> {
    ai_repository::get_or_create_default_config(&data.db).await
}

async fn check_token_limits(data: &Arc<AppState>, config: &AiConfig) -> Result<(), String> {
    if let Some(daily_limit) = config.daily_token_limit {
        let today = chrono::Utc::now().date_naive();
        let usage = ai_repository::get_token_usage_by_date(&data.db, today)
            .await
            .map_err(|_| "Failed to check token usage".to_string())?;
        let tokens_used_today = usage.map(|u| u.tokens_used).unwrap_or(0);
        if tokens_used_today >= daily_limit {
            return Err("Batas token harian telah tercapai".to_string());
        }
    }
    Ok(())
}

async fn log_ai_request(
    data: &Arc<AppState>,
    request: &GroqChatRequest,
    response: &GroqApiResponse,
    success: bool,
    error_message: Option<String>,
) -> Result<(), sqlx::Error> {
    let log = AiRequestLog {
        id: Uuid::new_v4(),
        prompt: request.prompt.clone(),
        response: if success && !response.choices.is_empty() {
            response.choices[0].message.content.clone()
        } else {
            "".to_string()
        },
        tokens_used: response.usage.total_tokens as i32,
        model: get_groq_model(),
        success,
        error_message,
        created_at: Utc::now(),
    };
    ai_repository::insert_ai_request_log(&data.db, &log).await?;
    Ok(())
}

async fn update_token_usage(data: &Arc<AppState>, tokens_used: i32) -> Result<(), sqlx::Error> {
    let today = chrono::Utc::now().date_naive();
    ai_repository::update_or_insert_token_usage(&data.db, today, tokens_used).await?;
    Ok(())
}

async fn validate_user_input(data: &Arc<AppState>, input: &str) -> Result<(), String> {
    let control = ai_repository::get_latest_user_input_controls(&data.db)
        .await
        .map_err(|_| "Failed to get input control settings".to_string())?;
    if let Some(control) = control {
        let input_lower = input.to_lowercase();
        if input.len() > control.max_input_length as usize {
            return Err(format!(
                "Input too long. Maximum {} characters allowed.",
                control.max_input_length
            ));
        }
        for blocked_keyword in &control.blocked_keywords {
            let keyword_lower = blocked_keyword.to_lowercase();
            let pattern = format!(r"\b{}\b", regex::escape(&keyword_lower));
            if let Ok(regex) = regex::Regex::new(&pattern) {
                if regex.is_match(&input_lower) {
                    return Err(format!(
                        "Input contains blocked keyword: {}",
                        blocked_keyword
                    ));
                }
            } else {
                let words: Vec<&str> = input_lower.split_whitespace().collect();
                if words.iter().any(|word| word == &keyword_lower) {
                    return Err(format!(
                        "Input contains blocked keyword: {}",
                        blocked_keyword
                    ));
                }
            }
        }
        if !control.required_keywords.is_empty() {
            let has_required_keyword = control
                .required_keywords
                .iter()
                .any(|keyword| input_lower.contains(&keyword.to_lowercase()));
            if !has_required_keyword {
                return Err(format!(
                    "Input must contain at least one of these keywords: {}",
                    control.required_keywords.join(", ")
                ));
            }
        }
    }
    validate_business_topics(input)?;
    Ok(())
}

// Advanced topic validation function
fn validate_business_topics(input: &str) -> Result<(), String> {
    let input_lower = input.to_lowercase();

    // Define allowed business topics with comprehensive keywords
    let pos_keywords = vec![
        "pos",
        "point of sale",
        "kasir",
        "pembayaran",
        "transaksi",
        "struk",
        "receipt",
        "barcode",
        "scanner",
        "cash register",
        "payment",
        "transaction",
        "checkout",
        "terminal",
        "mesin kasir",
        "sistem kasir",
        "penjualan langsung",
        "retail",
    ];

    let trends_keywords = vec![
        "trend",
        "trending",
        "popular",
        "populer",
        "tren",
        "penjualan terbaik",
        "best seller",
        "top selling",
        "hot item",
        "fast moving",
        "demand tinggi",
        "permintaan tinggi",
        "analisis trend",
        "trend analysis",
        "market trend",
        "tren pasar",
        "seasonal trend",
        "musiman",
        "pola penjualan",
        "sales pattern",
        "consumer behavior",
        "perilaku konsumen",
    ];

    // ID: Kata kunci kategori minuman untuk memperluas cakupan topik valid.
    // EN: Beverage category keywords to broaden valid topic coverage.
    let beverage_keywords = vec![
        "minuman",
        "beverage",
        "drink",
        "kopi",
        "coffee",
        "teh",
        "tea",
        "boba",
        "susu",
        "milk",
        "kombucha",
        "matcha",
        "jus",
        "juice",
        "soda",
        "isotonik",
        "energy drink",
        "yoghurt",
        "probiotik",
        "low sugar",
        "rendah gula",
        "ready to drink",
        "rtd",
    ];

    let weather_keywords = vec![
        "cuaca",
        "weather",
        "iklim",
        "climate",
        "hujan",
        "rain",
        "panas",
        "hot",
        "dingin",
        "cold",
        "musim",
        "season",
        "temperature",
        "suhu",
        "humidity",
        "kelembaban",
        "forecast",
        "ramalan cuaca",
        "prediksi cuaca",
        "weather prediction",
        "seasonal impact",
        "dampak musim",
        "weather influence",
        "pengaruh cuaca",
        "climate effect",
    ];

    let sales_app_keywords = vec![
        "aplikasi penjualan",
        "sales app",
        "inventory",
        "inventori",
        "stok",
        "stock",
        "produk",
        "product",
        "barang",
        "item",
        "katalog",
        "catalog",
        "harga",
        "price",
        "diskon",
        "discount",
        "promo",
        "promotion",
        "laporan",
        "report",
        "dashboard",
        "analytics",
        "analitik",
        "penjualan",
        "sales",
        "omzet",
        "revenue",
        "profit",
        "laba",
        "rugi",
        "loss",
        "margin",
        "customer",
        "pelanggan",
        "supplier",
        "pemasok",
        "order",
        "pesanan",
        "invoice",
        "faktur",
        "purchase",
        "pembelian",
        "procurement",
    ];

    let business_management_keywords = vec![
        "manajemen",
        "management",
        "bisnis",
        "business",
        "operasional",
        "operational",
        "keuangan",
        "finance",
        "accounting",
        "akuntansi",
        "budget",
        "anggaran",
        "cash flow",
        "arus kas",
        "roi",
        "return on investment",
        "kpi",
        "performance",
        "produktivitas",
        "productivity",
        "efisiensi",
        "efficiency",
        "strategi",
        "strategy",
    ];

    let bmc_keywords = vec![
        "bmc",
        "business model canvas",
        "model bisnis kanvas",
        "kanvas model bisnis",
        "value proposition",
        "nilai proposisi",
        "customer segment",
        "segmen pelanggan",
        "channels",
        "saluran",
        "revenue streams",
        "arus pendapatan",
        "cost structure",
        "struktur biaya",
        "key activities",
        "aktivitas utama",
        "key resources",
        "sumber daya utama",
        "key partners",
        "mitra utama",
    ];

    // Combine all allowed keywords
    let mut all_allowed_keywords = Vec::new();
    all_allowed_keywords.extend(pos_keywords);
    all_allowed_keywords.extend(trends_keywords);
    all_allowed_keywords.extend(weather_keywords);
    all_allowed_keywords.extend(sales_app_keywords);
    all_allowed_keywords.extend(business_management_keywords);
    all_allowed_keywords.extend(bmc_keywords);
    // ID: Tambahkan kata kunci minuman agar pertanyaan tren minuman lolos validasi.
    // EN: Add beverage keywords so beverage trend questions pass validation.
    all_allowed_keywords.extend(beverage_keywords);

    // Check if input contains at least one allowed keyword
    let has_business_keyword = all_allowed_keywords
        .iter()
        .any(|keyword| input_lower.contains(keyword));

    if !has_business_keyword {
        return Err(format!(
            "Maaf, saya hanya dapat membantu dengan topik yang berkaitan dengan:\n\
            • Point of Sale (POS) dan sistem kasir\n\
            • Analisis trend dan pola penjualan\n\
            • Pengaruh cuaca terhadap bisnis\n\
            • Aplikasi penjualan dan manajemen inventory\n\
            • Manajemen bisnis dan operasional\n\
            • Perencanaan bisnis (Business Model Canvas)\n\n\
            Silakan ajukan pertanyaan yang berkaitan dengan topik-topik tersebut."
        ));
    }

    // Define strictly prohibited topics
    let prohibited_topics = vec![
        "politik",
        "political",
        "agama",
        "religion",
        "sara",
        "pornografi",
        "porn",
        "kekerasan",
        "violence",
        "narkoba",
        "drugs",
        "gambling",
        "judi",
        "hack",
        "hacking",
        "virus",
        "malware",
        "illegal",
        "ilegal",
        "penipuan",
        "scam",
        "fraud",
        "personal data",
        "data pribadi",
        "password",
        "credit card",
        "kartu kredit",
        "bank account",
        "rekening bank",
        "social security",
        "nomor identitas",
    ];

    // Check for prohibited topics using word boundaries to avoid false positives
    for prohibited in &prohibited_topics {
        // Create regex pattern with word boundaries
        let pattern = format!(r"\b{}\b", regex::escape(prohibited));
        if let Ok(regex) = regex::Regex::new(&pattern) {
            if regex.is_match(&input_lower) {
                return Err(format!(
                    "Maaf, saya tidak dapat membantu dengan topik '{}'. \
                    Silakan fokus pada pertanyaan bisnis, POS, trend penjualan, atau cuaca.",
                    prohibited
                ));
            }
        } else {
            // Fallback to simple contains check if regex fails, but with additional validation
            if input_lower.contains(prohibited) {
                // Additional check to avoid false positives for common substrings
                let words: Vec<&str> = input_lower.split_whitespace().collect();
                if words.iter().any(|word| word == prohibited) {
                    return Err(format!(
                        "Maaf, saya tidak dapat membantu dengan topik '{}'. \
                        Silakan fokus pada pertanyaan bisnis, POS, trend penjualan, atau cuaca.",
                        prohibited
                    ));
                }
            }
        }
    }

    // Additional validation for business context
    validate_business_context(&input_lower)?;

    Ok(())
}

// Validate that the input is in proper business context
fn validate_business_context(input_lower: &str) -> Result<(), String> {
    // Check for non-business questions that might slip through
    let non_business_patterns = vec![
        "bagaimana cara membuat",
        "how to make",
        "resep",
        "recipe",
        "memasak",
        "cooking",
        "film",
        "movie",
        "musik",
        "music",
        "game",
        "permainan",
        "olahraga",
        "sports",
        "kesehatan",
        "health",
        "medis",
        "medical",
        "obat",
        "medicine",
        "dokter",
        "doctor",
        "sekolah",
        "school",
        "universitas",
        "university",
        "pelajaran",
        "lesson",
        "homework",
        "pr",
        "tugas sekolah",
        "ujian",
        "exam",
        "test",
    ];

    // If input contains non-business patterns without business context, reject
    for pattern in &non_business_patterns {
        if input_lower.contains(pattern) {
            // Check if it also contains business keywords
            let business_context_keywords = vec![
                "bisnis",
                "business",
                "penjualan",
                "sales",
                "toko",
                "store",
                "retail",
                "pos",
                "kasir",
                "inventory",
                "stok",
                "profit",
                "laba",
                "customer",
                "pelanggan",
            ];

            let has_business_context = business_context_keywords
                .iter()
                .any(|keyword| input_lower.contains(keyword));

            if !has_business_context {
                return Err(format!(
                    "Pertanyaan Anda tampaknya tidak berkaitan dengan bisnis atau aplikasi penjualan. \
                    Silakan ajukan pertanyaan yang berkaitan dengan POS, trend penjualan, cuaca bisnis, \
                    atau manajemen inventory."
                ));
            }
        }
    }

    Ok(())
}

// Rate limit helpers
async fn check_rate_limit(data: &Arc<AppState>, user_ip: &str) -> Result<bool, sqlx::Error> {
    let minute_window = Utc::now()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();
    let count = ai_repository::get_user_rate_limit_count(&data.db, user_ip, minute_window).await?;
    let control = ai_repository::get_latest_user_input_controls(&data.db).await?;
    let limit = control.map(|c| c.rate_limit_per_minute).unwrap_or(60);
    Ok(count < limit as i64)
}

async fn increment_rate_limit(data: &Arc<AppState>, user_ip: &str) -> Result<(), sqlx::Error> {
    let minute_window = Utc::now()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();
    ai_repository::increment_user_rate_limit(&data.db, user_ip, minute_window).await
}

// GROQ request helper
async fn make_groq_request(body: &GroqChatRequest) -> Result<GroqApiResponse, String> {
    let api_key = get_groq_api_key();
    let api_url = get_groq_api_url();
    let model = get_groq_model();

    // ID: Deteksi intent tren minuman dan sisipkan system prompt terarah.
    // EN: Detect beverage trend intent and inject a guiding system prompt.
    let prompt_lower = body.prompt.to_lowercase();
    let is_beverage_intent = {
        let trend_words = [
            "trend",
            "tren",
            "popular",
            "populer",
            "best seller",
            "top selling",
        ];
        let bev_words = [
            "minuman",
            "beverage",
            "drink",
            "kopi",
            "coffee",
            "teh",
            "tea",
            "boba",
            "susu",
            "milk",
            "kombucha",
            "matcha",
            "jus",
            "juice",
            "soda",
            "isotonik",
            "energy drink",
        ];
        trend_words.iter().any(|w| prompt_lower.contains(w))
            && bev_words.iter().any(|w| prompt_lower.contains(w))
    };

    let system_beverage_prompt = "Anda adalah analis pasar minuman untuk Indonesia. Jawab ringkas, faktual, dan terstruktur dengan poin-poin.\nFormat: \n- Kategori populer\n- Profil rasa & kesehatan\n- Kemasan & kanal distribusi\n- Rentang harga\n- Faktor musiman/cuaca\n- Rekomendasi aksi\nHindari klaim waktu real-time; gunakan tren umum dan regional jika relevan.";

    let req = if is_beverage_intent {
        GroqApiRequest {
            messages: vec![
                GroqMessage {
                    role: "system".to_string(),
                    content: system_beverage_prompt.to_string(),
                },
                GroqMessage {
                    role: "user".to_string(),
                    content: body.prompt.clone(),
                },
            ],
            model,
            max_tokens: body.max_tokens,
            temperature: body.temperature,
        }
    } else {
        GroqApiRequest {
            messages: vec![GroqMessage {
                role: "user".to_string(),
                content: body.prompt.clone(),
            }],
            model,
            max_tokens: body.max_tokens,
            temperature: body.temperature,
        }
    };

    let client = Client::new();
    let resp = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&req)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Groq API error: {}", resp.status()));
    }

    resp.json::<GroqApiResponse>()
        .await
        .map_err(|e| e.to_string())
}

// Token remaining helper
async fn calculate_remaining_tokens(data: &Arc<AppState>, config: &AiConfig) -> Option<u32> {
    if let Some(limit) = config.daily_token_limit {
        let today = chrono::Utc::now().date_naive();
        if let Ok(usage_opt) = ai_repository::get_token_usage_by_date(&data.db, today).await {
            let used = usage_opt.map(|u| u.tokens_used).unwrap_or(0);
            let remaining = (limit - used).max(0) as u32;
            return Some(remaining);
        }
        return Some(limit as u32);
    }
    None
}

// Detailed token usage endpoint
pub async fn get_detailed_token_usage(
    State(data): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<DetailedTokenUsageResponse>>, StatusCode> {
    let today = chrono::Utc::now().date_naive();
    let usage = ai_repository::get_token_usage_by_date(&data.db, today)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let config = get_ai_config(&data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tokens_used_today = usage.as_ref().map(|u| u.tokens_used).unwrap_or(0) as u32;
    let requests_count_today = usage.as_ref().map(|u| u.requests_count).unwrap_or(0) as u32;
    let avg_tokens_per_req = if requests_count_today > 0 {
        tokens_used_today as f32 / requests_count_today as f32
    } else {
        0.0
    };

    let daily_limit = config.daily_token_limit.map(|l| l as u32);
    let tokens_remaining = daily_limit.map(|limit| {
        if tokens_used_today >= limit {
            0
        } else {
            limit - tokens_used_today
        }
    });
    let usage_percentage =
        daily_limit.map(|limit| (tokens_used_today as f32 / limit as f32) * 100.0);
    let estimated_requests_remaining = match (daily_limit, avg_tokens_per_req) {
        (Some(limit), avg) if avg > 0.0 => {
            Some(((limit - tokens_used_today) as f32 / avg).floor() as u32)
        }
        _ => None,
    };

    let response = DetailedTokenUsageResponse {
        total_tokens_used_today: tokens_used_today,
        tokens_remaining,
        daily_limit,
        requests_count_today,
        last_reset: today.to_string(),
        usage_percentage,
        estimated_requests_remaining,
        average_tokens_per_request: avg_tokens_per_req,
    };

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Detailed token usage retrieved successfully".to_string(),
        data: response,
        errors: serde_json::json!({}),
    }))
}

// Token usage history endpoint
pub async fn get_token_usage_history(
    State(data): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<TokenUsageHistoryResponse>>>, StatusCode> {
    let history = ai_repository::get_token_usage_history(&data.db, 7)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let config = get_ai_config(&data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = history
        .into_iter()
        .map(|h| TokenUsageHistoryResponse {
            date: h.date.to_string(),
            tokens_used: h.tokens_used as u32,
            requests_count: h.requests_count as u32,
            daily_limit: config.daily_token_limit.map(|l| l as u32),
        })
        .collect();

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Token usage history retrieved successfully".to_string(),
        data: result,
        errors: serde_json::json!({}),
    }))
}

// Token monitoring alerts endpoint
pub async fn get_token_monitoring_alerts(
    State(data): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<TokenMonitoringAlert>>>, StatusCode> {
    let today = chrono::Utc::now().date_naive();
    let usage = ai_repository::get_token_usage_by_date(&data.db, today)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let config = get_ai_config(&data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut alerts: Vec<TokenMonitoringAlert> = Vec::new();
    if let Some(limit) = config.daily_token_limit {
        let used = usage.as_ref().map(|u| u.tokens_used).unwrap_or(0) as u32;
        let remaining = if used >= limit as u32 {
            0
        } else {
            limit as u32 - used
        };
        let percentage = (used as f32 / limit as f32) * 100.0;

        if percentage >= 100.0 {
            alerts.push(TokenMonitoringAlert {
                alert_type: "limit_reached".to_string(),
                message: "Daily token limit reached".to_string(),
                tokens_used: used,
                tokens_remaining: Some(0),
                percentage_used: Some(percentage),
            });
        } else if percentage >= 90.0 {
            alerts.push(TokenMonitoringAlert {
                alert_type: "warning".to_string(),
                message: "Token usage exceeds 90% of daily limit".to_string(),
                tokens_used: used,
                tokens_remaining: Some(remaining),
                percentage_used: Some(percentage),
            });
        }
    }

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "Token monitoring alerts retrieved successfully".to_string(),
        data: alerts,
        errors: serde_json::json!({}),
    }))
}

// User input controls endpoints
pub async fn get_user_input_controls(
    State(data): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<UserInputControlResponse>>, StatusCode> {
    let control = ai_repository::get_latest_user_input_controls(&data.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Placeholder IP; ideally extracted from request
    let user_ip = "127.0.0.1";
    let minute_window = Utc::now()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();
    let req_count = ai_repository::get_user_rate_limit_count(&data.db, user_ip, minute_window)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? as u32;

    let default_max_len = 1000u32;
    let default_rate = 60u32;
    let resp = match control {
        Some(c) => UserInputControlResponse {
            max_input_length: c.max_input_length as u32,
            rate_limit_per_minute: c.rate_limit_per_minute as u32,
            blocked_keywords: c.blocked_keywords,
            required_keywords: c.required_keywords,
            current_user_requests_this_minute: req_count,
        },
        None => UserInputControlResponse {
            max_input_length: default_max_len,
            rate_limit_per_minute: default_rate,
            blocked_keywords: vec![],
            required_keywords: vec![],
            current_user_requests_this_minute: req_count,
        },
    };

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "User input controls retrieved successfully".to_string(),
        data: resp,
        errors: serde_json::json!({}),
    }))
}

pub async fn update_user_input_controls(
    State(data): State<Arc<AppState>>,
    Json(body): Json<UserInputControlRequest>,
) -> Result<Json<ApiResponse<UserInputControlResponse>>, StatusCode> {
    let current = ai_repository::get_latest_user_input_controls(&data.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let max_input_length = body
        .max_input_length
        .map(|v| v as i32)
        .or_else(|| current.as_ref().map(|c| c.max_input_length))
        .unwrap_or(1000);
    let rate_limit_per_minute = body
        .rate_limit_per_minute
        .map(|v| v as i32)
        .or_else(|| current.as_ref().map(|c| c.rate_limit_per_minute))
        .unwrap_or(60);
    let blocked_keywords = body
        .blocked_keywords
        .or_else(|| current.as_ref().map(|c| c.blocked_keywords.clone()))
        .unwrap_or_default();
    let required_keywords = body
        .required_keywords
        .or_else(|| current.as_ref().map(|c| c.required_keywords.clone()))
        .unwrap_or_default();

    ai_repository::insert_user_input_controls(
        &data.db,
        max_input_length,
        rate_limit_per_minute,
        blocked_keywords.clone(),
        required_keywords.clone(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Placeholder IP; ideally extracted from request
    let user_ip = "127.0.0.1";
    let minute_window = Utc::now()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();
    let req_count = ai_repository::get_user_rate_limit_count(&data.db, user_ip, minute_window)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? as u32;

    let resp = UserInputControlResponse {
        max_input_length: max_input_length as u32,
        rate_limit_per_minute: rate_limit_per_minute as u32,
        blocked_keywords,
        required_keywords,
        current_user_requests_this_minute: req_count,
    };

    Ok(Json(ApiResponse {
        code: 200,
        status: "success".to_string(),
        message: "User input controls updated successfully".to_string(),
        data: resp,
        errors: serde_json::json!({}),
    }))
}
