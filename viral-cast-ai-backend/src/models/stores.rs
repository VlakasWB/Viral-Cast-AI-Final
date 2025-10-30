use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct StoresModel {
    pub uuid: Uuid,
    pub name: String,
    pub brand_url: Option<String>,
    pub province_code: Option<String>,
    pub regency_code: Option<String>,
    pub district_code: Option<String>,
    pub village_code: Option<String>,
    pub rt: Option<String>,
    pub rw: Option<String>,
    pub postal_code: Option<String>,
    pub telp: Option<String>,
    pub whatsapp: Option<String>,
    pub instagram: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}
