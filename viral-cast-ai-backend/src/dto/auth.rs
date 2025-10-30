use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedRegister {
    pub access_token: String,
    pub user: FilteredRegisterResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedLogin {
    pub access_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FilteredRegisterResponse {
    pub uuid: Uuid,
    pub username: String,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FilteredUserResponse {
    pub uuid: Uuid,
    pub username: String,
    pub email: Option<String>,
}
