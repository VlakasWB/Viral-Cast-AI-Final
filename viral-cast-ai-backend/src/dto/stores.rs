use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateStoreSchema {
    pub name: Option<String>,
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateStoreSchema {
    pub name: Option<String>,
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
    // Waktu toko sebagai Unix timestamp (milliseconds since epoch)
    // kirim nilai null untuk mengosongkan (set NULL)
    pub opening_time: Option<i64>,
    pub middle_closing_time: Option<i64>,
    pub closing_time: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct GetStoreSchema {
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
    pub opening_time: Option<i64>,
    pub middle_closing_time: Option<i64>,
    pub closing_time: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessedStore {
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
    pub opening_time: Option<i64>,
    pub middle_closing_time: Option<i64>,
    pub closing_time: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}
