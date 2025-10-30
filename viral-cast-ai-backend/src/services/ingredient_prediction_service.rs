// ID: Service untuk menangani prediksi bahan baku dengan rate limiting
// EN: Service for handling ingredient predictions with rate limiting

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::dto::groq::{GroqMessage, GroqRequest, GroqResponse};
use crate::handlers::store_ingredient_predictions::{
    IngredientPredictionResponse, LLMIngredientRecommendation, DEFAULT_MAX_PRODUCTS,
    MAX_INGREDIENTS_PER_BATCH, GROQ_MAX_ATTEMPTS,
};
use crate::models::store_ingredient_predictions::StoreIngredientPrediction;
use crate::repository::{
    ingredients as ingredients_repository, products as products_repository,
    regions as regions_repository, store_ingredient_predictions as predictions_repository,
    stores as stores_repository,
};
use crate::services::rate_limiter::GroqRateLimiter;
use crate::AppState;

pub struct IngredientPredictionService {
    state: Arc<AppState>,
    rate_limiter: GroqRateLimiter,
}

impl IngredientPredictionService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            rate_limiter: GroqRateLimiter::new(),
        }
    }

    pub async fn generate_predictions_for_store(
        &self,
        store_uuid: Uuid,
        max_products: Option<i32>,
    ) -> Result<Vec<IngredientPredictionResponse>, String> {
        let max_products = max_products.unwrap_or(DEFAULT_MAX_PRODUCTS);

        info!(
            store_uuid = %store_uuid,
            max_products = max_products,
            "Starting ingredient prediction generation"
        );

        // ID: Ambil data store
        // EN: Get store data
        let store = stores_repository::get_store_by_uuid(&self.state.db, store_uuid)
            .await
            .map_err(|e| format!("Failed to get store: {}", e))?
            .ok_or_else(|| "Store not found".to_string())?;

        // ID: Ambil konteks wilayah
        // EN: Get region context
        let region_context = if let Some(region_uuid) = store.region_uuid {
            regions_repository::get_region_by_uuid(&self.state.db, region_uuid)
                .await
                .map_err(|e| format!("Failed to get region: {}", e))?
                .map(|r| format!("Wilayah: {}, Provinsi: {}", r.name, r.province))
                .unwrap_or_else(|| "Wilayah tidak diketahui".to_string())
        } else {
            "Wilayah tidak diketahui".to_string()
        };

        // ID: Ambil produk toko
        // EN: Get store products
        let products = products_repository::get_products_by_store_uuid(&self.state.db, store_uuid)
            .await
            .map_err(|e| format!("Failed to get products: {}", e))?;

        if products.is_empty() {
            return Err("No products found for store".to_string());
        }

        let limited_products: Vec<_> = products.into_iter().take(max_products as usize).collect();

        // ID: Ambil semua bahan baku
        // EN: Get all ingredients
        let ingredients = ingredients_repository::get_all_ingredients(&self.state.db)
            .await
            .map_err(|e| format!("Failed to get ingredients: {}", e))?;

        if ingredients.is_empty() {
            return Err("No ingredients found".to_string());
        }

        // ID: Bagi bahan baku menjadi batch kecil
        // EN: Split ingredients into small batches
        let ingredient_chunks: Vec<_> = ingredients
            .chunks(MAX_INGREDIENTS_PER_BATCH)
            .collect();

        info!(
            total_ingredients = ingredients.len(),
            batch_count = ingredient_chunks.len(),
            batch_size = MAX_INGREDIENTS_PER_BATCH,
            "Split ingredients into batches"
        );

        let mut all_predictions = Vec::new();

        // ID: Proses setiap batch dengan delay
        // EN: Process each batch with delay
        for (batch_index, ingredient_batch) in ingredient_chunks.iter().enumerate() {
            // ID: Tambahkan delay antar batch (kecuali batch pertama)
            // EN: Add delay between batches (except first batch)
            if batch_index > 0 {
                let batch_delay_secs = std::env::var("GROQ_BATCH_DELAY_SECS")
                    .unwrap_or_else(|_| "15".to_string())
                    .parse::<u64>()
                    .unwrap_or(15);

                info!(
                    batch_index = batch_index,
                    delay_secs = batch_delay_secs,
                    "Waiting between batches to respect rate limits"
                );

                sleep(Duration::from_secs(batch_delay_secs)).await;
            }

            match self
                .process_ingredient_batch(
                    &store,
                    &region_context,
                    &limited_products,
                    ingredient_batch,
                    batch_index,
                )
                .await
            {
                Ok(mut batch_predictions) => {
                    all_predictions.append(&mut batch_predictions);
                }
                Err(e) => {
                    error!(
                        batch_index = batch_index,
                        error = %e,
                        "Failed to process ingredient batch"
                    );
                    // ID: Lanjutkan ke batch berikutnya
                    // EN: Continue to next batch
                    continue;
                }
            }
        }

        info!(
            total_predictions = all_predictions.len(),
            "Completed ingredient prediction generation"
        );

        Ok(all_predictions)
    }

    async fn process_ingredient_batch(
        &self,
        store: &crate::models::stores::Store,
        region_context: &str,
        products: &[crate::models::products::Product],
        ingredients: &[crate::models::ingredients::Ingredient],
        batch_index: usize,
    ) -> Result<Vec<IngredientPredictionResponse>, String> {
        // ID: Buat prompt untuk batch ini
        // EN: Create prompt for this batch
        let system_prompt = self.create_system_prompt(region_context);
        let user_prompt = self.create_user_prompt(store, products, ingredients);

        info!(
            batch_index = batch_index,
            ingredient_count = ingredients.len(),
            "Processing ingredient batch"
        );

        // ID: Panggil Groq API dengan rate limiting
        // EN: Call Groq API with rate limiting
        let predictions_model = std::env::var("GROQ_MODEL")
            .unwrap_or_else(|_| "llama-3.1-8b-instant".to_string());

        let llm_response = self
            .call_groq_with_model(
                vec![
                    GroqMessage {
                        role: "system".to_string(),
                        content: system_prompt.clone(),
                    },
                    GroqMessage {
                        role: "user".to_string(),
                        content: user_prompt.clone(),
                    },
                ],
                predictions_model.clone(),
                Some(4000),
                Some(0.3),
            )
            .await?;

        // ID: Parse respons LLM
        // EN: Parse LLM response
        let recommendations = self.parse_llm_response(&llm_response.content)?;

        // ID: Simpan prediksi ke database
        // EN: Save predictions to database
        let mut prediction_responses = Vec::new();

        for recommendation in recommendations {
            if let Some(ingredient) = ingredients
                .iter()
                .find(|i| i.name.to_lowercase() == recommendation.ingredient_name.to_lowercase())
            {
                let prediction = StoreIngredientPrediction {
                    uuid: Uuid::new_v4(),
                    store_uuid: store.uuid,
                    ingredient_uuid: ingredient.uuid,
                    predicted_quantity: recommendation.predicted_quantity,
                    confidence_score: recommendation.confidence_score,
                    reasoning: recommendation.reasoning.clone(),
                    llm_model: predictions_model.clone(),
                    llm_prompt: user_prompt.clone(),
                    llm_response: llm_response.content.clone(),
                    llm_reasoning: recommendation.reasoning.clone(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                match predictions_repository::create_store_ingredient_prediction(
                    &self.state.db,
                    &prediction,
                )
                .await
                {
                    Ok(_) => {
                        prediction_responses.push(IngredientPredictionResponse {
                            ingredient_uuid: ingredient.uuid,
                            ingredient_name: ingredient.name.clone(),
                            predicted_quantity: recommendation.predicted_quantity,
                            confidence_score: recommendation.confidence_score,
                            reasoning: recommendation.reasoning,
                        });
                    }
                    Err(e) => {
                        error!(
                            ingredient_name = %ingredient.name,
                            error = %e,
                            "Failed to save ingredient prediction"
                        );
                    }
                }
            }
        }

        Ok(prediction_responses)
    }

    async fn call_groq_with_model(
        &self,
        messages: Vec<GroqMessage>,
        model: String,
        max_tokens: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<GroqResponse, String> {
        let req_body = serde_json::to_string(&GroqRequest {
            model: model.clone(),
            messages,
            max_tokens,
            temperature,
        })
        .map_err(|e| format!("Failed to serialize request: {}", e))?;

        // ID: Buat kunci cache
        // EN: Create cache key
        let mut hasher = DefaultHasher::new();
        req_body.hash(&mut hasher);
        let cache_key = format!("groq_{}_{}", model, hasher.finish());

        // ID: Cek cache terlebih dahulu
        // EN: Check cache first
        if let Some(cached_response) = self.rate_limiter.get_cached_response(&cache_key).await {
            info!("Using cached Groq response");
            return Ok(cached_response);
        }

        // ID: Estimasi token untuk request ini
        // EN: Estimate tokens for this request
        let estimated_tokens = req_body.len() / 4 + max_tokens.unwrap_or(1000) as usize;

        // ID: Cek rate limit
        // EN: Check rate limit
        if !self.rate_limiter.can_make_request(estimated_tokens).await {
            let wait_time = self.rate_limiter.get_suggested_wait_time().await;
            return Err(format!(
                "Rate limit exceeded. Please wait {} seconds",
                wait_time
            ));
        }

        // ID: Lakukan request dengan retry
        // EN: Make request with retry
        let mut backoff_secs = 1;
        let mut last_error = String::new();

        for attempt in 1..=GROQ_MAX_ATTEMPTS {
            match self.make_groq_request(&req_body).await {
                Ok(response) => {
                    // ID: Catat penggunaan dan simpan ke cache
                    // EN: Record usage and save to cache
                    self.rate_limiter.record_request(estimated_tokens).await;
                    self.rate_limiter
                        .cache_response(cache_key, response.clone())
                        .await;

                    return Ok(response);
                }
                Err(e) => {
                    last_error = e.clone();
                    warn!(
                        attempt = attempt,
                        max_attempts = GROQ_MAX_ATTEMPTS,
                        error = %e,
                        "Groq API request failed"
                    );

                    if attempt < GROQ_MAX_ATTEMPTS {
                        info!(
                            backoff_secs = backoff_secs,
                            "Waiting before retry"
                        );
                        sleep(Duration::from_secs(backoff_secs)).await;
                        backoff_secs = (backoff_secs * 2).min(60);
                    }
                }
            }
        }

        Err(format!(
            "Groq rate limit reached after {} attempts. Last error: {}",
            GROQ_MAX_ATTEMPTS, last_error
        ))
    }

    async fn make_groq_request(&self, req_body: &str) -> Result<GroqResponse, String> {
        // ID: Implementasi request HTTP ke Groq API
        // EN: HTTP request implementation to Groq API
        // Note: Ini adalah placeholder - implementasi sebenarnya akan menggunakan reqwest
        // This is a placeholder - actual implementation would use reqwest
        
        // Untuk sementara, return error untuk menunjukkan bahwa ini perlu implementasi lengkap
        // For now, return error to show this needs complete implementation
        Err("HTTP client implementation needed".to_string())
    }

    fn create_system_prompt(&self, region_context: &str) -> String {
        format!(
            r#"Anda adalah asisten AI yang membantu memprediksi kebutuhan restock bahan baku untuk toko makanan dan minuman.

Konteks wilayah: {}

Tugas Anda:
1. Analisis produk yang dijual toko
2. Prediksi bahan baku yang perlu di-restock dalam 7 hari ke depan
3. Berikan estimasi jumlah yang dibutuhkan
4. Sertakan tingkat kepercayaan (0.0-1.0)
5. Berikan alasan untuk setiap prediksi

Format respons harus JSON array dengan struktur:
[
  {{
    "ingredient_name": "nama_bahan",
    "predicted_quantity": jumlah_prediksi,
    "confidence_score": skor_kepercayaan,
    "reasoning": "alasan_prediksi"
  }}
]

Pertimbangkan:
- Popularitas produk
- Musim dan cuaca
- Tren konsumsi lokal
- Efisiensi penggunaan bahan"#,
            region_context
        )
    }

    fn create_user_prompt(
        &self,
        store: &crate::models::stores::Store,
        products: &[crate::models::products::Product],
        ingredients: &[crate::models::ingredients::Ingredient],
    ) -> String {
        let products_list = products
            .iter()
            .map(|p| format!("- {} (Harga: Rp{})", p.name, p.price))
            .collect::<Vec<_>>()
            .join("\n");

        let ingredients_list = ingredients
            .iter()
            .map(|i| format!("- {} (Stok: {} {})", i.name, i.stock_quantity, i.unit))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Toko: {}
Alamat: {}

Produk yang dijual:
{}

Bahan baku yang tersedia:
{}

Prediksi kebutuhan restock untuk 7 hari ke depan:"#,
            store.name,
            store.address.as_deref().unwrap_or("Alamat tidak tersedia"),
            products_list,
            ingredients_list
        )
    }

    fn parse_llm_response(&self, content: &str) -> Result<Vec<LLMIngredientRecommendation>, String> {
        // ID: Parse respons JSON dari LLM
        // EN: Parse JSON response from LLM
        serde_json::from_str::<Vec<LLMIngredientRecommendation>>(content)
            .map_err(|e| format!("Failed to parse LLM response: {}", e))
    }

    pub async fn get_rate_limiter_stats(&self) -> crate::services::rate_limiter::RateLimiterStats {
        self.rate_limiter.get_stats().await
    }
}