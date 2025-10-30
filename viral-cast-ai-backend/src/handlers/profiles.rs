use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Extension;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::middleware::jwt::JWTAuthMiddleware;
use crate::{
    dto::api::ApiResponse,
    dto::profiles::{
        CreateProfileSchema, GetProfilesSchema, ProcessedProfile, Roles, UpdateProfileSchema,
    },
    models::profiles::ProfilesModel,
    AppState,
};

pub async fn create_profiles_handler(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateProfileSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_uuid = &jwtauth.user.uuid;

    // Safely check for existing profile without unwrap panics
    let existing_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM profiles WHERE user_uuid = $1")
            .bind(user_uuid)
            .fetch_one(&data.db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "code": 500,
                        "status": "INTERNAL_SERVER_ERROR",
                        "message": format!("Database error: {:?}", e),
                        "data": {},
                        "errors": {}
                    })),
                )
            })?;

    if existing_count > 0 {
        let error_response = ApiResponse {
            code: 409,
            status: "CONFLICT".to_string(),
            message: "Profiles sudah pernah dibuat".to_string(),
            data: {},
            errors: json!({}),
        };

        let error_response_json = json!({
            "code": error_response.code,
            "status": error_response.status,
            "message": error_response.message,
            "data": {},
            "errors": {},
        });

        return Err((StatusCode::CONFLICT, Json(json!(error_response_json))));
    }

    let query_result = sqlx::query_as::<_, ProfilesModel>(
        "INSERT INTO profiles (
                user_uuid,
                first_name,
                last_name,
                photo_profile,
                background_profile,
                gender,
                telp,
                birth_date,
                birth_place,
                roles_number,
                store_uuid,
                province_code,
                regency_code,
                district_code,
                village_code,
                rt,
                rw,
                postal_code)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, COALESCE($10, 6), $11, $12, $13, $14, $15, $16, $17, $18) RETURNING *"
    )
    .bind(user_uuid)
    .bind(body.first_name.to_owned().unwrap_or("".to_string()))
    .bind(body.last_name.to_owned().unwrap_or("".to_string()))
    .bind(body.photo_profile.to_owned().unwrap_or("".to_string()))
    .bind(body.background_profile.to_owned().unwrap_or("".to_string()))
    .bind(body.gender.to_owned().unwrap_or("".to_string()))
    .bind(body.telp.to_owned().unwrap_or("".to_string()))
    .bind(body.birth_date.to_owned().unwrap_or("".to_string()))
    .bind(body.birth_place.to_owned().unwrap_or("".to_string()))
    .bind(body.roles_number.map(|x| x as i32))
    .bind(body.store_uuid.to_owned())
    .bind(body.province_code.to_owned())
    .bind(body.regency_code.to_owned())
    .bind(body.district_code.to_owned())
    .bind(body.village_code.to_owned())
    .bind(body.rt.to_owned())
    .bind(body.rw.to_owned())
    .bind(body.postal_code.to_owned())
    .fetch_one(&data.db)
    .await;

    let get_profiles = sqlx::query_as::<_, GetProfilesSchema>(
        "SELECT
            A.uuid,
            A.user_uuid,
            A.first_name,
            A.last_name,
            A.photo_profile,
            A.background_profile,
            A.gender,
            A.telp,
            A.birth_date,
            A.birth_place,
            A.roles_number,
            I.name AS roles_name,
            A.store_uuid,
            A.province_code,
            A.regency_code,
            A.district_code,
            A.village_code,
            A.rt,
            A.rw,
            A.postal_code
        FROM profiles A
        LEFT JOIN roles I ON A.roles_number = I.number
        WHERE A.user_uuid = $1",
    )
    .bind(user_uuid)
    .fetch_all(&data.db)
    .await;

    let get_profile = get_profiles.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "status": "Internal Server Error",
                "message": "Failed to fetch profiles",
                "data": null,
                "errors": {},
            })),
        )
    })?;

    if get_profile.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "code": 404,
                "status": "Not Found",
                "message": "No profiles found for the given user UUID",
                "data": null,
                "errors": {},
            })),
        ));
    }

    let profile = get_profile.into_iter().next().unwrap();

    let processed_profile = ProcessedProfile {
        uuid: profile.uuid.clone(),
        user_uuid: profile.user_uuid,
        first_name: profile.first_name.clone(),
        last_name: profile.last_name.clone(),
        photo_profile: profile.photo_profile.clone(),
        background_profile: profile.background_profile.clone(),
        gender: profile.gender.clone(),
        telp: profile.telp.clone(),
        birth_date: profile.birth_date.clone(),
        birth_place: profile.birth_place.clone(),
        roles: Roles {
            number: profile.roles_number,
            name: profile.roles_name.clone(),
        },
        store_uuid: profile.store_uuid.clone(),
        province_code: profile.province_code.clone(),
        regency_code: profile.regency_code.clone(),
        district_code: profile.district_code.clone(),
        village_code: profile.village_code.clone(),
        rt: profile.rt.clone(),
        rw: profile.rw.clone(),
        postal_code: profile.postal_code.clone(),
    };

    match query_result {
        Ok(T) => {
            let response = ApiResponse {
                code: 201,
                status: "CREATED".to_string(),
                message: "Create profiles successfully".to_string(),
                data: processed_profile,
                errors: json!({}),
            };

            Ok((StatusCode::CREATED, Json(response)))
        }

        Err(e) => {
            let error_response = ApiResponse {
                code: 500,
                status: "INTERNAL_SERVER_ERROR".to_string(),
                message: format!("{:?}", e).to_string(),
                data: {},
                errors: json!({}),
            };

            let error_response_json = json!({
                "code": error_response.code,
                "status": error_response.status,
                "message": error_response.message,
                "data": {},
                "errors": {},
            });

            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response_json)))
        }
    }
}

pub async fn update_profiles_handler_put(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateProfileSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_uuid = &jwtauth.user.uuid;

    let query_result = sqlx::query_as::<_, ProfilesModel>(
        "SELECT
            uuid,
            user_uuid,
            first_name,
            last_name,
            photo_profile,
            background_profile,
            gender,
            telp,
            birth_date,
            birth_place,
            roles_number,
            store_uuid,
            province_code,
            regency_code,
            district_code,
            village_code,
            rt,
            rw,
            postal_code,
            created_at,
            updated_at,
            deleted_at
        FROM profiles WHERE user_uuid = $1",
    )
    .bind(user_uuid)
    .fetch_one(&data.db)
    .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "code": 404,
            "status": "NOT FOUND",
            "message": format!("Profile with ID: {} not found", user_uuid),
            "data": {},
            "errors": {},
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let now = SystemTime::now();

    let duration = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    let unix_timestamp_now = duration.as_secs() * 1000 + u64::from(duration.subsec_millis());
    let unix_timestamp_now_i64: i64 = unix_timestamp_now as i64;

    let profile = query_result.unwrap();

    let query_result = sqlx::query_as::<_, ProfilesModel>(
        "UPDATE profiles SET
            first_name = $1,
            last_name = $2,
            photo_profile = $3,
            background_profile = $4,
            gender = $5,
            telp = $6,
            birth_date = $7,
            birth_place = $8,
            roles_number = $9,
            store_uuid = $10,
            province_code = $11,
            regency_code = $12,
            district_code = $13,
            village_code = $14,
            rt = $15,
            rw = $16,
            postal_code = $17,
            updated_at = $18
        WHERE user_uuid = $19
        RETURNING *",
    )
    .bind(
        body.first_name
            .to_owned()
            .unwrap_or(profile.first_name.unwrap_or_default()),
    )
    .bind(
        body.last_name
            .to_owned()
            .unwrap_or(profile.last_name.unwrap_or_default()),
    )
    .bind(
        body.photo_profile
            .to_owned()
            .unwrap_or(profile.photo_profile.unwrap_or_default()),
    )
    .bind(
        body.background_profile
            .to_owned()
            .unwrap_or(profile.background_profile.unwrap_or_default()),
    )
    .bind(
        body.gender
            .to_owned()
            .unwrap_or(profile.gender.unwrap_or_default()),
    )
    .bind(
        body.telp
            .to_owned()
            .unwrap_or(profile.telp.unwrap_or_default()),
    )
    .bind(
        body.birth_date
            .to_owned()
            .unwrap_or(profile.birth_date.unwrap_or_default()),
    )
    .bind(
        body.birth_place
            .to_owned()
            .unwrap_or(profile.birth_place.unwrap_or_default()),
    )
    .bind(body.roles_number.to_owned().or(profile.roles_number))
    .bind(body.store_uuid.to_owned().or(profile.store_uuid))
    .bind(body.province_code.to_owned().or(profile.province_code))
    .bind(body.regency_code.to_owned().or(profile.regency_code))
    .bind(body.district_code.to_owned().or(profile.district_code))
    .bind(body.village_code.to_owned().or(profile.village_code))
    .bind(body.rt.to_owned().or(profile.rt))
    .bind(body.rw.to_owned().or(profile.rw))
    .bind(body.postal_code.to_owned().or(profile.postal_code))
    .bind(unix_timestamp_now_i64)
    .bind(user_uuid)
    .fetch_one(&data.db)
    .await;

    let get_profiles = sqlx::query_as::<_, GetProfilesSchema>(
        "SELECT
            A.uuid,
            A.user_uuid,
            A.first_name,
            A.last_name,
            A.photo_profile,
            A.background_profile,
            A.gender,
            A.telp,
            A.birth_date,
            A.birth_place,
            A.roles_number,
            I.name AS roles_name,
            A.store_uuid,
            A.province_code,
            A.regency_code,
            A.district_code,
            A.village_code,
            A.rt,
            A.rw,
            A.postal_code
        FROM profiles A
        LEFT JOIN roles I ON A.roles_number = I.number
        WHERE A.user_uuid = $1",
    )
    .bind(user_uuid)
    .fetch_all(&data.db)
    .await;

    let get_profile = get_profiles.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "status": "Internal Server Error",
                "message": "Failed to fetch profiles",
                "data": null,
                "errors": {},
            })),
        )
    })?;

    if get_profile.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "code": 404,
                "status": "Not Found",
                "message": "No profiles found for the given user UUID",
                "data": null,
                "errors": {},
            })),
        ));
    }

    let profile = get_profile.into_iter().next().unwrap();

    let processed_profile = ProcessedProfile {
        uuid: profile.uuid.clone(),
        user_uuid: profile.user_uuid,
        first_name: profile.first_name.clone(),
        last_name: profile.last_name.clone(),
        photo_profile: profile.photo_profile.clone(),
        background_profile: profile.background_profile.clone(),
        gender: profile.gender.clone(),
        telp: profile.telp.clone(),
        birth_date: profile.birth_date.clone(),
        birth_place: profile.birth_place.clone(),
        roles: Roles {
            number: profile.roles_number,
            name: profile.roles_name.clone(),
        },
        store_uuid: profile.store_uuid.clone(),
        province_code: profile.province_code.clone(),
        regency_code: profile.regency_code.clone(),
        district_code: profile.district_code.clone(),
        village_code: profile.village_code.clone(),
        rt: profile.rt.clone(),
        rw: profile.rw.clone(),
        postal_code: profile.postal_code.clone(),
    };

    match query_result {
        Ok(profile) => {
            let profile_response = ApiResponse {
                code: 200,
                status: "OK".to_string(),
                message: "Profile updated successfully".to_string(),
                data: processed_profile,
                errors: json!({}),
            };

            return Ok(Json(profile_response));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code": 500,
                    "status": "INTERNAL_SERVER_ERROR",
                    "message": format!("{:?}", err),
                    "data": {},
                    "errors": {},
                })),
            ));
        }
    }
}

pub async fn update_profiles_handler_patch(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateProfileSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_uuid = &jwtauth.user.uuid;

    let query_result = sqlx::query_as::<_, ProfilesModel>(
        "SELECT
            uuid,
            user_uuid,
            first_name,
            last_name,
            photo_profile,
            background_profile,
            gender,
            telp,
            birth_date,
            birth_place,
            roles_number,
            store_uuid,
            province_code,
            regency_code,
            district_code,
            village_code,
            rt,
            rw,
            postal_code,
            created_at,
            updated_at,
            deleted_at
        FROM profiles WHERE user_uuid = $1",
    )
    .bind(user_uuid)
    .fetch_one(&data.db)
    .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "code": 404,
            "status": "NOT FOUND",
            "message": format!("Profile with ID: {} not found", user_uuid),
            "data": {},
            "errors": {},
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let now = SystemTime::now();

    let duration = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    let unix_timestamp_now = duration.as_secs() * 1000 + u64::from(duration.subsec_millis());
    let unix_timestamp_now_i64: i64 = unix_timestamp_now as i64;

    let profile = query_result.unwrap();

    let query_result = sqlx::query_as::<_, ProfilesModel>(
        "UPDATE profiles SET
            first_name = $1,
            last_name = $2,
            photo_profile = $3,
            background_profile = $4,
            gender = $5,
            telp = $6,
            birth_date = $7,
            birth_place = $8,
            roles_number = $9,
            store_uuid = $10,
            province_code = $11,
            regency_code = $12,
            district_code = $13,
            village_code = $14,
            rt = $15,
            rw = $16,
            postal_code = $17,
            updated_at = $18
        WHERE user_uuid = $19
        RETURNING *",
    )
    .bind(
        body.first_name
            .to_owned()
            .unwrap_or(profile.first_name.unwrap_or_default()),
    )
    .bind(
        body.last_name
            .to_owned()
            .unwrap_or(profile.last_name.unwrap_or_default()),
    )
    .bind(
        body.photo_profile
            .to_owned()
            .unwrap_or(profile.photo_profile.unwrap_or_default()),
    )
    .bind(
        body.background_profile
            .to_owned()
            .unwrap_or(profile.background_profile.unwrap_or_default()),
    )
    .bind(
        body.gender
            .to_owned()
            .unwrap_or(profile.gender.unwrap_or_default()),
    )
    .bind(
        body.telp
            .to_owned()
            .unwrap_or(profile.telp.unwrap_or_default()),
    )
    .bind(
        body.birth_date
            .to_owned()
            .unwrap_or(profile.birth_date.unwrap_or_default()),
    )
    .bind(
        body.birth_place
            .to_owned()
            .unwrap_or(profile.birth_place.unwrap_or_default()),
    )
    .bind(body.roles_number.to_owned().or(profile.roles_number))
    .bind(body.store_uuid.to_owned().or(profile.store_uuid))
    .bind(body.province_code.to_owned().or(profile.province_code))
    .bind(body.regency_code.to_owned().or(profile.regency_code))
    .bind(body.district_code.to_owned().or(profile.district_code))
    .bind(body.village_code.to_owned().or(profile.village_code))
    .bind(body.rt.to_owned().or(profile.rt))
    .bind(body.rw.to_owned().or(profile.rw))
    .bind(body.postal_code.to_owned().or(profile.postal_code))
    .bind(unix_timestamp_now_i64)
    .bind(user_uuid)
    .fetch_one(&data.db)
    .await;

    let get_profiles = sqlx::query_as::<_, GetProfilesSchema>(
        "SELECT
            A.uuid,
            A.user_uuid,
            A.first_name,
            A.last_name,
            A.photo_profile,
            A.background_profile,
            A.gender,
            A.telp,
            A.birth_date,
            A.birth_place,
            A.roles_number,
            I.name AS roles_name,
            A.store_uuid,
            A.province_code,
            A.regency_code,
            A.district_code,
            A.village_code,
            A.rt,
            A.rw,
            A.postal_code
        FROM profiles A
        LEFT JOIN roles I ON A.roles_number = I.number
        WHERE A.user_uuid = $1",
    )
    .bind(user_uuid)
    .fetch_all(&data.db)
    .await;

    let get_profile = get_profiles.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "status": "Internal Server Error",
                "message": "Failed to fetch profiles",
                "data": null,
                "errors": {},
            })),
        )
    })?;

    if get_profile.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "code": 404,
                "status": "Not Found",
                "message": "No profiles found for the given user UUID",
                "data": null,
                "errors": {},
            })),
        ));
    }

    let profile = get_profile.into_iter().next().unwrap();

    let processed_profile = ProcessedProfile {
        uuid: profile.uuid.clone(),
        user_uuid: profile.user_uuid,
        first_name: profile.first_name.clone(),
        last_name: profile.last_name.clone(),
        photo_profile: profile.photo_profile.clone(),
        background_profile: profile.background_profile.clone(),
        gender: profile.gender.clone(),
        telp: profile.telp.clone(),
        birth_date: profile.birth_date.clone(),
        birth_place: profile.birth_place.clone(),
        roles: Roles {
            number: profile.roles_number,
            name: profile.roles_name.clone(),
        },
        store_uuid: profile.store_uuid.clone(),
        province_code: profile.province_code.clone(),
        regency_code: profile.regency_code.clone(),
        district_code: profile.district_code.clone(),
        village_code: profile.village_code.clone(),
        rt: profile.rt.clone(),
        rw: profile.rw.clone(),
        postal_code: profile.postal_code.clone(),
    };

    match query_result {
        Ok(_) => {
            let profile_response = ApiResponse {
                code: 200,
                status: "OK".to_string(),
                message: "Profile updated successfully".to_string(),
                data: processed_profile,
                errors: json!({}),
            };

            return Ok(Json(profile_response));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code": 500,
                    "status": "INTERNAL_SERVER_ERROR",
                    "message": format!("{:?}", err),
                    "data": {},
                    "errors": {},
                })),
            ));
        }
    }
}

pub async fn get_my_profiles_handler(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_uuid = &jwtauth.user.uuid;

    let get_profiles = sqlx::query_as::<_, GetProfilesSchema>(
        "SELECT
            A.uuid,
            A.user_uuid,
            A.first_name,
            A.last_name,
            A.photo_profile,
            A.background_profile,
            A.gender,
            A.telp,
            A.birth_date,
            A.birth_place,
            A.roles_number,
            I.name AS roles_name,
            A.store_uuid,
            A.province_code,
            A.regency_code,
            A.district_code,
            A.village_code,
            A.rt,
            A.rw,
            A.postal_code
        FROM profiles A
        LEFT JOIN roles I ON A.roles_number = I.number
        WHERE A.user_uuid = $1",
    )
    .bind(user_uuid)
    .fetch_all(&data.db)
    .await;

    let profiles = get_profiles.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "status": "Internal Server Error",
                "message": "Failed to fetch profiles",
                "data": null,
                "errors": {},
            })),
        )
    })?;

    if profiles.is_empty() {
        // Auto-create profile minimal ketika belum ada
        if let Err(e) = crate::repository::auth::create_profile_for_user(&data.db, *user_uuid).await
        {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code": 500,
                    "status": "Internal Server Error",
                    "message": format!("Gagal membuat profil otomatis: {:?}", e),
                    "data": null,
                    "errors": {},
                })),
            ));
        }

        // Ambil ulang profil setelah dibuat
        let profiles_after = sqlx::query_as::<_, GetProfilesSchema>(
            "SELECT
                A.uuid,
                A.user_uuid,
                A.first_name,
                A.last_name,
                A.photo_profile,
                A.background_profile,
                A.gender,
                A.telp,
                A.birth_date,
                A.birth_place,
                A.roles_number,
                I.name AS roles_name,
                A.store_uuid,
                A.province_code,
                A.regency_code,
                A.district_code,
                A.village_code,
                A.rt,
                A.rw,
                A.postal_code
            FROM profiles A
            LEFT JOIN roles I ON A.roles_number = I.number
            WHERE A.user_uuid = $1",
        )
        .bind(user_uuid)
        .fetch_all(&data.db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code": 500,
                    "status": "Internal Server Error",
                    "message": "Failed to fetch profiles after auto-create",
                    "data": null,
                    "errors": {},
                })),
            )
        })?;

        if profiles_after.is_empty() {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code": 500,
                    "status": "Internal Server Error",
                    "message": "Profil tidak ditemukan meskipun auto-create sukses",
                    "data": null,
                    "errors": {},
                })),
            ));
        }

        let profile = profiles_after.into_iter().next().unwrap();

        let processed_profile = ProcessedProfile {
            uuid: profile.uuid.clone(),
            user_uuid: profile.user_uuid,
            first_name: profile.first_name.clone(),
            last_name: profile.last_name.clone(),
            photo_profile: profile.photo_profile.clone(),
            background_profile: profile.background_profile.clone(),
            gender: profile.gender.clone(),
            telp: profile.telp.clone(),
            birth_date: profile.birth_date.clone(),
            birth_place: profile.birth_place.clone(),
            roles: Roles {
                number: profile.roles_number,
                name: profile.roles_name.clone(),
            },
            store_uuid: profile.store_uuid.clone(),
            province_code: profile.province_code.clone(),
            regency_code: profile.regency_code.clone(),
            district_code: profile.district_code.clone(),
            village_code: profile.village_code.clone(),
            rt: profile.rt.clone(),
            rw: profile.rw.clone(),
            postal_code: profile.postal_code.clone(),
        };

        let response = ApiResponse {
            code: 200,
            status: "OK".to_string(),
            message: "Profile auto-created".to_string(),
            data: processed_profile,
            errors: json!({}),
        };

        return Ok(Json(response));
    }

    let profile = profiles.into_iter().next().unwrap();

    let processed_profile = ProcessedProfile {
        uuid: profile.uuid.clone(),
        user_uuid: profile.user_uuid,
        first_name: profile.first_name.clone(),
        last_name: profile.last_name.clone(),
        photo_profile: profile.photo_profile.clone(),
        background_profile: profile.background_profile.clone(),
        gender: profile.gender.clone(),
        telp: profile.telp.clone(),
        birth_date: profile.birth_date.clone(),
        birth_place: profile.birth_place.clone(),
        roles: Roles {
            number: profile.roles_number,
            name: profile.roles_name.clone(),
        },
        store_uuid: profile.store_uuid.clone(),
        province_code: profile.province_code.clone(),
        regency_code: profile.regency_code.clone(),
        district_code: profile.district_code.clone(),
        village_code: profile.village_code.clone(),
        rt: profile.rt.clone(),
        rw: profile.rw.clone(),
        postal_code: profile.postal_code.clone(),
    };

    let response = ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: "Get data profile successfully".to_string(),
        data: processed_profile,
        errors: json!({}),
    };

    Ok(Json(response))
}

pub async fn get_user_profiles_handler(
    Path(user_uuid): Path<Uuid>,
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let get_profiles = sqlx::query_as::<_, GetProfilesSchema>(
        "SELECT
            A.uuid,
            A.user_uuid,
            A.first_name,
            A.last_name,
            A.photo_profile,
            A.background_profile,
            A.gender,
            A.telp,
            A.birth_date,
            A.birth_place,
            A.roles_number,
            I.name AS roles_name,
            A.store_uuid,
            A.province_code,
            A.regency_code,
            A.district_code,
            A.village_code,
            A.rt,
            A.rw,
            A.postal_code
        FROM profiles A
        LEFT JOIN roles I ON A.roles_number = I.number
        WHERE A.user_uuid = $1",
    )
    .bind(user_uuid)
    .fetch_all(&data.db)
    .await;

    let profiles = get_profiles.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "status": "Internal Server Error",
                "message": "Failed to fetch profiles",
                "data": null,
                "errors": {},
            })),
        )
    })?;

    if profiles.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "code": 404,
                "status": "Not Found",
                "message": "No profiles found for the given user UUID",
                "data": null,
                "errors": {},
            })),
        ));
    }

    let profile = profiles.into_iter().next().unwrap();

    let processed_profile = ProcessedProfile {
        uuid: profile.uuid.clone(),
        user_uuid: profile.user_uuid,
        first_name: profile.first_name.clone(),
        last_name: profile.last_name.clone(),
        photo_profile: profile.photo_profile.clone(),
        background_profile: profile.background_profile.clone(),
        gender: profile.gender.clone(),
        telp: profile.telp.clone(),
        birth_date: profile.birth_date.clone(),
        birth_place: profile.birth_place.clone(),
        roles: Roles {
            number: profile.roles_number,
            name: profile.roles_name.clone(),
        },
        store_uuid: profile.store_uuid.clone(),
        province_code: profile.province_code.clone(),
        regency_code: profile.regency_code.clone(),
        district_code: profile.district_code.clone(),
        village_code: profile.village_code.clone(),
        rt: profile.rt.clone(),
        rw: profile.rw.clone(),
        postal_code: profile.postal_code.clone(),
    };

    let response = ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: "Get data profile successfully".to_string(),
        data: processed_profile,
        errors: json!({}),
    };

    Ok(Json(response))
}
