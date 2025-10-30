use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};

use crate::dto::regions::{
    DistrictDetailQuery, DistrictDetailResponse, DistrictListRequest, DistrictListResponse,
    PaginationInfo, ProvinceDetailQuery, ProvinceDetailResponse, ProvinceListRequest,
    ProvinceListResponse, RegencyDetailQuery, RegencyDetailResponse, RegencyListRequest,
    RegencyListResponse, VillageDetailQuery, VillageDetailResponse, VillageListRequest,
    VillageListResponse,
};
use crate::models::regions::{DistrictView, ProvinceView, RegencyView, VillageView};
use crate::repository::regions as regions_repository;
use crate::AppState;

// Province handlers
pub async fn get_provinces(
    Query(params): Query<ProvinceListRequest>,
    State(state): State<AppState>,
) -> Result<Json<ProvinceListResponse>, StatusCode> {
    match regions_repository::list_provinces(&state.db, &params).await {
        Ok((provinces, total, limit, offset)) => {
            let pagination = PaginationInfo {
                total,
                limit,
                offset,
                has_next: offset + limit < total,
            };
            Ok(Json(ProvinceListResponse {
                success: true,
                message: "Province list retrieved successfully".to_string(),
                data: provinces,
                pagination,
            }))
        }
        Err(e) => {
            eprintln!("Error executing province query: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_province_by_code(
    Path(code): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ProvinceDetailResponse>, StatusCode> {
    match regions_repository::get_province_by_code(&state.db, &code).await {
        Ok(Some(province)) => Ok(Json(ProvinceDetailResponse {
            success: true,
            message: "Province detail retrieved successfully".to_string(),
            data: Some(province),
        })),
        Ok(None) => Ok(Json(ProvinceDetailResponse {
            success: false,
            message: "Province not found".to_string(),
            data: None,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Province detail by code or name
pub async fn get_province_detail(
    Query(params): Query<ProvinceDetailQuery>,
    State(state): State<AppState>,
) -> Result<Json<ProvinceDetailResponse>, StatusCode> {
    if params.code.is_none() && params.name.is_none() {
        return Ok(Json(ProvinceDetailResponse {
            success: false,
            message: "Parameter 'code' or 'name' is required".to_string(),
            data: None,
        }));
    }

    match regions_repository::get_province_detail(&state.db, &params).await {
        Ok(Some(province)) => Ok(Json(ProvinceDetailResponse {
            success: true,
            message: "Province detail retrieved successfully".to_string(),
            data: Some(province),
        })),
        Ok(None) => Ok(Json(ProvinceDetailResponse {
            success: false,
            message: "Province not found".to_string(),
            data: None,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Regency handlers
pub async fn get_regencies(
    Query(params): Query<RegencyListRequest>,
    State(state): State<AppState>,
) -> Result<Json<RegencyListResponse>, StatusCode> {
    match regions_repository::list_regencies(&state.db, &params).await {
        Ok((regencies, total, limit, offset)) => {
            let pagination = PaginationInfo {
                total,
                limit,
                offset,
                has_next: offset + limit < total,
            };
            Ok(Json(RegencyListResponse {
                success: true,
                message: "Regency/City list retrieved successfully".to_string(),
                data: regencies,
                pagination,
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_regency_by_code(
    Path(code): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<RegencyDetailResponse>, StatusCode> {
    match regions_repository::get_regency_by_code(&state.db, &code).await {
        Ok(Some(regency)) => Ok(Json(RegencyDetailResponse {
            success: true,
            message: "Regency/City detail retrieved successfully".to_string(),
            data: Some(regency),
        })),
        Ok(None) => Ok(Json(RegencyDetailResponse {
            success: false,
            message: "Regency/City not found".to_string(),
            data: None,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Regency detail by code or name
pub async fn get_regency_detail(
    Query(params): Query<RegencyDetailQuery>,
    State(state): State<AppState>,
) -> Result<Json<RegencyDetailResponse>, StatusCode> {
    if params
        .code
        .as_deref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(false)
        && params
            .name
            .as_deref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(false)
    {
        return Ok(Json(RegencyDetailResponse {
            success: false,
            message: "Parameter 'code' or 'name' is required".to_string(),
            data: None,
        }));
    }

    match regions_repository::get_regency_detail(&state.db, &params).await {
        Ok(Some(regency)) => Ok(Json(RegencyDetailResponse {
            success: true,
            message: "Regency/City detail retrieved successfully".to_string(),
            data: Some(regency),
        })),
        Ok(None) => Ok(Json(RegencyDetailResponse {
            success: false,
            message: "Regency/City not found".to_string(),
            data: None,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// District handlers
pub async fn get_districts(
    Query(params): Query<DistrictListRequest>,
    State(state): State<AppState>,
) -> Result<Json<DistrictListResponse>, StatusCode> {
    match regions_repository::list_districts(&state.db, &params).await {
        Ok((districts, total, limit, offset)) => {
            let pagination = PaginationInfo {
                total,
                limit,
                offset,
                has_next: offset + limit < total,
            };
            Ok(Json(DistrictListResponse {
                success: true,
                message: "District list retrieved successfully".to_string(),
                data: districts,
                pagination,
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_district_by_code(
    Path(code): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DistrictDetailResponse>, StatusCode> {
    match regions_repository::get_district_by_code(&state.db, &code).await {
        Ok(Some(district)) => Ok(Json(DistrictDetailResponse {
            success: true,
            message: "District detail retrieved successfully".to_string(),
            data: Some(district),
        })),
        Ok(None) => Ok(Json(DistrictDetailResponse {
            success: false,
            message: "District not found".to_string(),
            data: None,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// District detail by code or name
pub async fn get_district_detail(
    Query(params): Query<DistrictDetailQuery>,
    State(state): State<AppState>,
) -> Result<Json<DistrictDetailResponse>, StatusCode> {
    if params
        .code
        .as_deref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(false)
        && params
            .name
            .as_deref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(false)
    {
        return Ok(Json(DistrictDetailResponse {
            success: false,
            message: "Parameter 'code' or 'name' is required".to_string(),
            data: None,
        }));
    }

    match regions_repository::get_district_detail(&state.db, &params).await {
        Ok(Some(district)) => Ok(Json(DistrictDetailResponse {
            success: true,
            message: "District detail retrieved successfully".to_string(),
            data: Some(district),
        })),
        Ok(None) => Ok(Json(DistrictDetailResponse {
            success: false,
            message: "District not found".to_string(),
            data: None,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Village handlers
pub async fn get_villages(
    Query(params): Query<VillageListRequest>,
    State(state): State<AppState>,
) -> Result<Json<VillageListResponse>, StatusCode> {
    match regions_repository::list_villages(&state.db, &params).await {
        Ok((villages, total, limit, offset)) => {
            let pagination = PaginationInfo {
                total,
                limit,
                offset,
                has_next: offset + limit < total,
            };
            Ok(Json(VillageListResponse {
                success: true,
                message: "Village list retrieved successfully".to_string(),
                data: villages,
                pagination,
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_village_by_code(
    Path(code): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<VillageDetailResponse>, StatusCode> {
    match regions_repository::get_village_by_code(&state.db, &code).await {
        Ok(Some(village)) => Ok(Json(VillageDetailResponse {
            success: true,
            message: "Village detail retrieved successfully".to_string(),
            data: Some(village),
        })),
        Ok(None) => Ok(Json(VillageDetailResponse {
            success: false,
            message: "Village not found".to_string(),
            data: None,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Village detail by code or name
pub async fn get_village_detail(
    Query(params): Query<VillageDetailQuery>,
    State(state): State<AppState>,
) -> Result<Json<VillageDetailResponse>, StatusCode> {
    if params
        .code
        .as_deref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(false)
        && params
            .name
            .as_deref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(false)
    {
        return Ok(Json(VillageDetailResponse {
            success: false,
            message: "Parameter 'code' or 'name' is required".to_string(),
            data: None,
        }));
    }

    match regions_repository::get_village_detail(&state.db, &params).await {
        Ok(Some(village)) => Ok(Json(VillageDetailResponse {
            success: true,
            message: "Village detail retrieved successfully".to_string(),
            data: Some(village),
        })),
        Ok(None) => Ok(Json(VillageDetailResponse {
            success: false,
            message: "Village not found".to_string(),
            data: None,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
