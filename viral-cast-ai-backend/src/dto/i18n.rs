use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum I18nEntity {
    IngredientCatalog,
    Product,
    Category,
    RecipeSet,
    RecipeItem,
    IngredientStockMove,
    IngredientStock,
    IngredientMarketPrice,
}

fn validate_locale(value: &str) -> Result<(), ValidationError> {
    match value.to_lowercase().as_str() {
        "id" | "en" => Ok(()),
        _ => Err(ValidationError::new("invalid_locale")),
    }
}

fn validate_status(value: &str) -> Result<(), ValidationError> {
    match value.to_uppercase().as_str() {
        "PENDING" | "COMPLETED" | "REVIEW_REQUIRED" => Ok(()),
        _ => Err(ValidationError::new("invalid_translation_status")),
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ResolveTranslationQuery {
    pub entity: I18nEntity,
    pub uuid: Uuid,
    #[validate(custom(function = "validate_locale"))]
    pub locale: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpsertTranslationSchema {
    pub entity: I18nEntity,
    pub uuid: Uuid,
    #[validate(custom(function = "validate_locale"))]
    pub locale: String,
    #[validate(custom(function = "validate_status"))]
    pub translation_status: Option<String>,
    // Common translatable fields across entities (optional per entity)
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub notes: Option<String>,
    // Special for ingredient_stocks
    pub ingredient_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TranslationResultResponse {
    pub entity: String,
    pub uuid: Uuid,
    pub locale: String,
    pub fields: serde_json::Value,
}
