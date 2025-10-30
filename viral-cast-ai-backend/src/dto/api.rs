use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Debug)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub status: String,
    pub message: String,
    pub data: T,
    pub errors: serde_json::Value,
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug, Default)]
pub struct FilterDateRangeOptions {
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Pagination {
    pub current_page: i64,
    pub total_pages: i64,
    pub next_page: Option<i64>,
    pub prev_page: Option<i64>,
    pub total_available_records: i64,
    pub total_displayed_records: i64,
    pub total_remaining_records: i64,
}

impl<T: serde::Serialize> ApiResponse<T> {
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
