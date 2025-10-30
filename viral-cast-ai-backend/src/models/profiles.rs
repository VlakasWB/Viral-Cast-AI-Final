use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, FromRow)]
pub struct ProfilesModel {
    pub uuid: Uuid,
    pub user_uuid: Option<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub photo_profile: Option<String>,
    pub background_profile: Option<String>,
    pub gender: Option<String>,
    pub telp: Option<String>,
    pub birth_date: Option<String>,
    pub birth_place: Option<String>,
    pub roles_number: Option<i32>,
    pub store_uuid: Option<Uuid>,
    // Region codes (nullable)
    pub province_code: Option<String>,
    pub regency_code: Option<String>,
    pub district_code: Option<String>,
    pub village_code: Option<String>,
    // Neighborhood codes
    pub rt: Option<String>,
    pub rw: Option<String>,
    pub postal_code: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}
