use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct GoogleAdsSearchRequest {
    #[validate(length(min = 3, message = "Query minimal 3 karakter"))]
    pub query: String,
    pub page_size: Option<u32>,
    pub customer_id: Option<String>,
}
