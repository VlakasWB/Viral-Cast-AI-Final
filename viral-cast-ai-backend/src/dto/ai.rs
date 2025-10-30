use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroqChatRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroqChatUnlimitedRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub bypass_validation: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroqChatResponse {
    pub response: String,
    pub tokens_used: u32,
    pub tokens_remaining: Option<u32>,
    pub model: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroqApiRequest {
    pub messages: Vec<GroqMessage>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

// ID: Tambahkan Clone karena Vec<GroqMessage> perlu Clone ketika dicopy.
// EN: Add Clone because Vec<GroqMessage> needs Clone when being cloned.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroqMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroqApiResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<GroqChoice>,
    pub usage: GroqUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroqChoice {
    pub index: u32,
    pub message: GroqMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroqUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AiConfigResponse {
    pub input_validation_enabled: bool,
    pub token_limit_enabled: bool,
    pub max_input_length: u32,
    pub allowed_topics: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateAiConfigRequest {
    pub input_validation_enabled: Option<bool>,
    pub token_limit_enabled: Option<bool>,
    pub max_input_length: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenUsageResponse {
    pub total_tokens_used_today: u32,
    pub tokens_remaining: Option<u32>,
    pub daily_limit: Option<u32>,
    pub last_reset: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DetailedTokenUsageResponse {
    pub total_tokens_used_today: u32,
    pub tokens_remaining: Option<u32>,
    pub daily_limit: Option<u32>,
    pub requests_count_today: u32,
    pub last_reset: String,
    pub usage_percentage: Option<f32>,
    pub estimated_requests_remaining: Option<u32>,
    pub average_tokens_per_request: f32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenUsageHistoryResponse {
    pub date: String,
    pub tokens_used: u32,
    pub requests_count: u32,
    pub daily_limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserInputControlRequest {
    pub max_input_length: Option<u32>,
    pub rate_limit_per_minute: Option<u32>,
    pub blocked_keywords: Option<Vec<String>>,
    pub required_keywords: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserInputControlResponse {
    pub max_input_length: u32,
    pub rate_limit_per_minute: u32,
    pub blocked_keywords: Vec<String>,
    pub required_keywords: Vec<String>,
    pub current_user_requests_this_minute: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenMonitoringAlert {
    pub alert_type: String, // "warning" | "critical" | "limit_reached"
    pub message: String,
    pub tokens_used: u32,
    pub tokens_remaining: Option<u32>,
    pub percentage_used: Option<f32>,
}
