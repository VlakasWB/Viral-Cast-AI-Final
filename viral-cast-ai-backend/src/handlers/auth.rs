use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, Response, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};

use rand_core::OsRng;
use redis::AsyncCommands;
use serde_json::{json, to_string};

use crate::repository::auth as auth_repository;
use crate::{
    dto::{
        api::ApiResponse,
        auth::{FilteredRegisterResponse, FilteredUserResponse, ProcessedLogin, ProcessedRegister},
        auth_token::{self, TokenDetails},
        users::FilteredUser,
    },
    middleware::jwt::JWTAuthMiddleware,
    models::user::{LoginUserSchema, RegisterUserSchema, User},
    AppState,
};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Viral Cast AI API Version 1 Safe and Healthy";

    let json_response = serde_json::json!({
        "code": 200,
        "status": "OK",
        "message": MESSAGE,
    });

    Json(json_response)
}

pub async fn register_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let username_exists = auth_repository::username_exists(&data.db, &body.username)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Database error: {}", e),
                "data": {},
                "errors": {},
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    if username_exists {
        let error_response = serde_json::json!({
            "code": 409,
            "status": "CONFLICT",
            "message": "Username already exists",
            "data": {},
            "errors": {},
        });
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    // Only check email if it's provided
    if let Some(ref email) = body.email {
        let email_exists = auth_repository::email_exists(&data.db, email)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "code": 500,
                    "status": "INTERNAL_SERVER_ERROR",
                    "message": format!("Database error: {}", e),
                    "data": {},
                    "errors": {},
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;
        if email_exists {
            let error_response = serde_json::json!({
                "code": 409,
                "status": "CONFLICT",
                "message": "Email already exists",
                "data": {},
                "errors": {},
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Password hashing error: {}", e),
                "data": {},
                "errors": {},
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?
        .to_string();
    let hashed_password = password_hash;

    let user = auth_repository::insert_user(
        &data.db,
        &body.username,
        body.email.as_deref(),
        &hashed_password,
    )
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "code": 500,
            "status": "INTERNAL_SERVER_ERROR",
            "message": format!("Database error: {}", e),
            "data": {},
            "errors": {},
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    // Automatically create a profile for the newly registered user
    auth_repository::create_profile_for_user(&data.db, user.uuid)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Failed to auto-create profile: {}", e),
                "data": {},
                "errors": {},
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let access_token_details = generate_token(
        user.uuid,
        data.env.access_token_max_age,
        data.env.access_token_private_key.to_owned(),
    )?;
    let refresh_token_details = generate_token(
        user.uuid,
        data.env.refresh_token_max_age,
        data.env.refresh_token_private_key.to_owned(),
    )?;

    save_token_data_to_redis(&data, &access_token_details, data.env.access_token_max_age).await?;
    save_token_data_to_redis(
        &data,
        &refresh_token_details,
        data.env.refresh_token_max_age,
    )
    .await?;

    let access_cookie = Cookie::build((
        "access_token",
        access_token_details.token.clone().unwrap_or_default(),
    ))
    .path("/")
    .max_age(time::Duration::minutes(data.env.access_token_max_age * 60))
    .same_site(SameSite::Lax)
    .http_only(true);

    let refresh_cookie = Cookie::build((
        "refresh_token",
        refresh_token_details.token.unwrap_or_default(),
    ))
    .path("/")
    .max_age(time::Duration::minutes(data.env.refresh_token_max_age * 60))
    .same_site(SameSite::Lax)
    .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(data.env.access_token_max_age * 60))
        .same_site(SameSite::Lax)
        .http_only(false);

    let mut processed_register = ProcessedRegister {
        access_token: access_token_details.token.unwrap(),
        user: FilteredRegisterResponse {
            uuid: user.uuid,
            username: user.username,
            email: user.email.clone(),
        },
    };

    let user_response = ApiResponse {
        code: 201,
        status: "CREATED".to_string(),
        message: "Register user successfully".to_string(),
        data: processed_register,
        errors: json!({}),
    };

    let mut headers = HeaderMap::new();

    let access_header = header::HeaderValue::from_str(&access_cookie.to_string()).map_err(|e| {
        let error_response = serde_json::json!({
            "code": 500,
            "status": "INTERNAL_SERVER_ERROR",
            "message": format!("Failed to set access_token cookie: {}", e),
            "data": {},
            "errors": {},
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;
    headers.append(header::SET_COOKIE, access_header);

    let refresh_header =
        header::HeaderValue::from_str(&refresh_cookie.to_string()).map_err(|e| {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Failed to set refresh_token cookie: {}", e),
                "data": {},
                "errors": {},
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    headers.append(header::SET_COOKIE, refresh_header);

    let logged_in_header =
        header::HeaderValue::from_str(&logged_in_cookie.to_string()).map_err(|e| {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Failed to set logged_in cookie: {}", e),
                "data": {},
                "errors": {},
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    headers.append(header::SET_COOKIE, logged_in_header);

    let mut response_builder = Response::builder().status(StatusCode::CREATED);

    for (key, value) in headers.iter() {
        response_builder = response_builder.header(key, value);
    }
    // Pastikan semua response JSON memiliki header Content-Type
    response_builder = response_builder.header(header::CONTENT_TYPE, "application/json");

    // Serialisasi ApiResponse menjadi JSON string
    let user_response_json = to_string(&user_response).unwrap();

    // Pembuatan respons dengan JSON string sebagai body
    let response = response_builder
        .body(Body::from(user_response_json))
        .unwrap();

    Ok(response)
}

pub async fn login_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user = auth_repository::find_user_by_username(&data.db, &body.username)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Database error: {}", e),
                "data": {},
                "errors": {},
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?
        .ok_or_else(|| {
            let error_response = serde_json::json!({
                "code": 400,
                "status": "BAD_REQUEST",
                "message": "Username atau password salah",
                "data": {},
                "errors": {},
            });
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        let error_response = serde_json::json!({
            "code": 400,
            "status": "BAD_REQUEST",
            "message": "Username atau password salah",
            "data": {},
            "errors": {},
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let access_token_details = generate_token(
        user.uuid,
        data.env.access_token_max_age,
        data.env.access_token_private_key.to_owned(),
    )?;
    let refresh_token_details = generate_token(
        user.uuid,
        data.env.refresh_token_max_age,
        data.env.refresh_token_private_key.to_owned(),
    )?;

    save_token_data_to_redis(&data, &access_token_details, data.env.access_token_max_age).await?;
    save_token_data_to_redis(
        &data,
        &refresh_token_details,
        data.env.refresh_token_max_age * 60,
    )
    .await?;

    let access_cookie = Cookie::build((
        "access_token",
        access_token_details.token.clone().unwrap_or_default(),
    ))
    .path("/")
    .max_age(time::Duration::minutes(data.env.access_token_max_age * 60))
    .same_site(SameSite::Lax)
    .http_only(true);

    let refresh_cookie = Cookie::build((
        "refresh_token",
        refresh_token_details.token.unwrap_or_default(),
    ))
    .path("/")
    .max_age(time::Duration::minutes(data.env.refresh_token_max_age * 60))
    .same_site(SameSite::Lax)
    .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(data.env.access_token_max_age * 60))
        .same_site(SameSite::Lax)
        .http_only(false);

    let processed_login = ProcessedLogin {
        access_token: access_token_details.token.clone(),
    };

    let login_response = ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: "Login success".to_string(),
        data: processed_login,
        errors: json!({}),
    };

    let login_response_json = to_string(&login_response).unwrap();
    let mut response = Response::new(login_response_json);

    let mut headers = HeaderMap::new();

    let access_header = header::HeaderValue::from_str(&access_cookie.to_string()).map_err(|e| {
        let error_response = serde_json::json!({
            "code": 500,
            "status": "INTERNAL_SERVER_ERROR",
            "message": format!("Failed to set access_token cookie: {}", e),
            "data": {},
            "errors": {},
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;
    headers.append(header::SET_COOKIE, access_header);

    let refresh_header =
        header::HeaderValue::from_str(&refresh_cookie.to_string()).map_err(|e| {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Failed to set refresh_token cookie: {}", e),
                "data": {},
                "errors": {},
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    headers.append(header::SET_COOKIE, refresh_header);

    let logged_in_header =
        header::HeaderValue::from_str(&logged_in_cookie.to_string()).map_err(|e| {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Failed to set logged_in cookie: {}", e),
                "data": {},
                "errors": {},
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    headers.append(header::SET_COOKIE, logged_in_header);

    response.headers_mut().extend(headers);
    // Pastikan semua response JSON memiliki header Content-Type
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    Ok(response)
}

pub async fn refresh_access_token_handler(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let message = "Could not refresh access token";

    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| {
            let error_response = serde_json::json!({
                "code": 403,
                "status": "FORBIDDEN",
                "message": message,
                "data": {},
                "errors": {},
            });
            (StatusCode::FORBIDDEN, Json(error_response))
        })?;

    let refresh_token_details = match auth_token::verify_jwt_token(
        data.env.refresh_token_public_key.to_owned(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            let error_response = serde_json::json!({
                "code": 401,
                "status": "UNAUTHORIZED",
                "message": format_args!("{:?}", e),
                "data": {},
                "errors": {},
            });
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    // Try Redis, fallback to in-memory session store
    let redis_token_user_uuid_opt = match data.redis_client.get_multiplexed_async_connection().await
    {
        Ok(mut conn) => {
            match conn
                .get::<_, String>(refresh_token_details.token_uuid.to_string())
                .await
            {
                Ok(v) => Some(v),
                Err(_) => None,
            }
        }
        Err(_) => None,
    };

    let redis_token_user_uuid = if let Some(v) = redis_token_user_uuid_opt {
        v
    } else {
        let store = data.session_store.lock().await;
        if let Some(v) = store
            .get(&refresh_token_details.token_uuid.to_string())
            .cloned()
        {
            v
        } else {
            let error_response = serde_json::json!({
                "code": 401,
                "status": "UNAUTHORIZED",
                "message": "Token is invalid or session has expired",
                "data": {},
                "errors": {},
            });
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    let user_uuid_uuid = uuid::Uuid::parse_str(&redis_token_user_uuid).map_err(|_| {
        let error_response = serde_json::json!({
            "code": 401,
            "status": "UNAUTHORIZED",
            "message": "Token is invalid or session has expired",
            "data": {},
            "errors": {},
        });
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

    let user = sqlx::query_as!(
        User,
        "SELECT uuid, username, email, password, access_token, refresh_token, created_at, updated_at, deleted_at FROM users WHERE uuid = $1", 
        user_uuid_uuid
    )
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Error fetching user from database: {}", e),
                "data": {},
                "errors": {},
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let user = user.ok_or_else(|| {
        let error_response = serde_json::json!({
            "code": 401,
            "status": "UNAUTHORIZED",
            "message": "The user belonging to this token no longer exists".to_string(),
            "data": {},
            "errors": {},
        });
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

    let access_token_details = generate_token(
        user.uuid,
        data.env.access_token_max_age,
        data.env.access_token_private_key.to_owned(),
    )?;

    save_token_data_to_redis(
        &data,
        &access_token_details,
        data.env.access_token_max_age * 60,
    )
    .await?;

    let access_cookie = Cookie::build((
        "access_token",
        access_token_details.token.clone().unwrap_or_default(),
    ))
    .path("/")
    .max_age(time::Duration::minutes(data.env.access_token_max_age * 60))
    .same_site(SameSite::Lax)
    .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(data.env.access_token_max_age * 60))
        .same_site(SameSite::Lax)
        .http_only(false);

    let processed_refresh = ProcessedLogin {
        access_token: access_token_details.token.clone(),
    };

    let refresh_response = ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: "Refresh access token success".to_string(),
        data: processed_refresh,
        errors: json!({}),
    };

    let refresh_response_json = to_string(&refresh_response).unwrap();
    let mut response = Response::new(refresh_response_json);

    let mut headers = HeaderMap::new();

    let access_header = header::HeaderValue::from_str(&access_cookie.to_string()).map_err(|e| {
        let error_response = serde_json::json!({
            "code": 500,
            "status": "INTERNAL_SERVER_ERROR",
            "message": format!("Failed to set access_token cookie: {}", e),
            "data": {},
            "errors": {},
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;
    headers.append(header::SET_COOKIE, access_header);

    let logged_in_header =
        header::HeaderValue::from_str(&logged_in_cookie.to_string()).map_err(|e| {
            let error_response = serde_json::json!({
                "code": 500,
                "status": "INTERNAL_SERVER_ERROR",
                "message": format!("Failed to set logged_in cookie: {}", e),
                "data": {},
                "errors": {},
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    headers.append(header::SET_COOKIE, logged_in_header);

    *response.headers_mut() = headers;
    // Pastikan semua response JSON memiliki header Content-Type
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    Ok(response)
}

pub async fn logout_handler(
    cookie_jar: CookieJar,
    Extension(auth_guard): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let message = "Token is invalid or session has expired";

    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| {
            let error_response = json!({
                "code": 403,
                "status": "FORBIDDEN",
                "message": message,
                "data": {},
                "errors": {},
            });
            (StatusCode::FORBIDDEN, Json(error_response))
        })?;

    let refresh_token_details = match auth_token::verify_jwt_token(
        data.env.refresh_token_public_key.to_owned(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            let error_response = json!({
                "code": 401,
                "status": "UNAUTHORIZED",
                "message": format!("{:?}", e),
                "data": {},
                "errors": {},
            });
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    // Try to delete from Redis, fallback to in-memory store
    if let Ok(mut conn) = data.redis_client.get_multiplexed_async_connection().await {
        let _ = conn
            .del::<_, ()>(&[
                refresh_token_details.token_uuid.to_string(),
                auth_guard.access_token_uuid.to_string(),
            ])
            .await;
    } else {
        let mut store = data.session_store.lock().await;
        store.remove(&refresh_token_details.token_uuid.to_string());
        store.remove(&auth_guard.access_token_uuid.to_string());
    }

    let access_cookie = Cookie::build("access_token")
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();
    let refresh_cookie = Cookie::build("refresh_token")
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();

    let logged_in_cookie = Cookie::build("logged_in")
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(false)
        .finish();

    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        logged_in_cookie.to_string().parse().unwrap(),
    );

    let logout_response = ApiResponse::<serde_json::Value> {
        code: 200,
        status: "OK".to_string(),
        message: "Logout success".to_string(),
        data: serde_json::json!({}),
        errors: serde_json::json!({}),
    };

    let mut response = Response::new(to_string(&logout_response).unwrap());
    *response.headers_mut() = headers;
    // Pastikan semua response JSON memiliki header Content-Type
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    Ok(response)
}

pub async fn get_me_handler(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_uuid = &jwtauth.user.uuid;

    let get_users = sqlx::query_as!(
        FilteredUserResponse,
        "SELECT
            A.uuid,
            A.email,
            A.username
        FROM users A
        WHERE A.uuid = $1",
        user_uuid,
    )
    .fetch_all(&data.db)
    .await;

    let users = get_users.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "status": "Internal Server Error",
                "message": "Failed to fetch user",
                "data": null,
                "errors": {},
            })),
        )
    })?;

    if users.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "code": 404,
                "status": "Not Found",
                "message": "No user found for the given user UUID",
                "data": null,
                "errors": {},
            })),
        ));
    }

    let user = users.into_iter().next().unwrap();

    let processed_user = FilteredUser {
        uuid: user.uuid.to_string(),
        email: user.email.clone(),
        username: user.username.to_owned(),
    };

    let json_response = ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: "Get data user successfully".to_string(),
        data: processed_user,
        errors: json!({}),
    };

    Ok(Json(json_response))
}

fn filter_user_record(user: &User) -> FilteredUser {
    FilteredUser {
        uuid: user.uuid.to_string(),
        email: user.email.clone(),
        username: user.username.to_owned(),
        // access_token: user.access_token.to_owned().unwrap_or("".to_string()),
        // refresh_token: user.refresh_token.to_owned().unwrap_or("".to_string()),
        // created_at: user.created_at.unwrap(),
        // updated_at: user.updated_at.unwrap(),
        // deleted_at: user.deleted_at.unwrap_or(0),
    }
}

fn generate_token(
    user_uuid: uuid::Uuid,
    max_age: i64,
    private_key: String,
) -> Result<TokenDetails, (StatusCode, Json<serde_json::Value>)> {
    // Trace key characteristics to diagnose InvalidKeyFormat issues
    let preview: String = private_key.chars().take(32).collect();
    tracing::info!(
        "generate_token: user={}, max_age={}, key_len={}, key_prefix='{}'",
        user_uuid,
        max_age,
        private_key.len(),
        preview
    );

    auth_token::generate_jwt_token(user_uuid, max_age, private_key).map_err(|e| {
        let error_response = serde_json::json!({
            "code": 500,
            "status": "INTERNAL_SERVER_ERROR",
            "message": format!("error generating token: {}", e),
            "data": {},
            "errors": {},
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })
}

async fn save_token_data_to_redis(
    data: &Arc<AppState>,
    token_details: &TokenDetails,
    max_age: i64,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    // Try Redis first; on any error, fall back to in-memory session store
    match data.redis_client.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            if let Err(e) = conn
                .set_ex::<_, _, ()>(
                    token_details.token_uuid.to_string(),
                    token_details.user_uuid.to_string(),
                    (max_age) as u64,
                )
                .await
            {
                // Fallback on set error
                let mut store = data.session_store.lock().await;
                store.insert(
                    token_details.token_uuid.to_string(),
                    token_details.user_uuid.to_string(),
                );
                tracing::warn!("Redis set_ex failed, using in-memory store: {:?}", e);
            }
            Ok(())
        }
        Err(e) => {
            let mut store = data.session_store.lock().await;
            store.insert(
                token_details.token_uuid.to_string(),
                token_details.user_uuid.to_string(),
            );
            tracing::warn!("Redis connection failed, using in-memory store: {:?}", e);
            Ok(())
        }
    }
}
