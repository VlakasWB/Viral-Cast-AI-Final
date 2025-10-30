use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct UnitsOfMeasureModel {
    pub uuid: Uuid,
    pub code: Option<String>,
    pub name: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub deleted_at: Option<i64>,
}
