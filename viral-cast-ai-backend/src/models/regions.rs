use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Province {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Regency {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
    pub province_uuid: Uuid,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct District {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
    pub regency_uuid: Uuid,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Village {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
    pub district_uuid: Uuid,
}

// View models with hierarchical information
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProvinceView {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RegencyView {
    pub code: String,
    pub name: String,
    pub province_code: String,
    pub province_name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DistrictView {
    pub code: String,
    pub name: String,
    pub regency_code: String,
    pub regency_name: String,
    pub province_code: String,
    pub province_name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct VillageView {
    pub code: String,
    pub name: String,
    pub district_code: String,
    pub district_name: String,
    pub regency_code: String,
    pub regency_name: String,
    pub province_code: String,
    pub province_name: String,
}
