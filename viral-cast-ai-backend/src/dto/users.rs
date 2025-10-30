use serde::Serialize;

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct FilteredUser {
    pub uuid: String,
    pub username: String,
    pub email: Option<String>,
    // pub access_token: String,
    // pub refresh_token: String,
    // pub created_at: Option<i64>,
    // pub updated_at: Option<i64>,
    // pub deleted_at: Option<i64>,
}

#[derive(Serialize, Debug)]
pub struct UserData {
    pub user: FilteredUser,
}

#[derive(Serialize, Debug)]
pub struct UserResponse {
    pub status: String,
    pub data: UserData,
}
