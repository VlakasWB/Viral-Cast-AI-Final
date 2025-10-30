use crate::models::regions::{DistrictView, ProvinceView, RegencyView, VillageView};
use serde::{Deserialize, Serialize};

// Request DTOs
#[derive(Debug, Deserialize)]
pub struct ProvinceListRequest {
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Detail query DTOs (search by code or name)
#[derive(Debug, Deserialize)]
pub struct ProvinceDetailQuery {
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegencyListRequest {
    pub province_code: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RegencyDetailQuery {
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DistrictListRequest {
    pub regency_code: Option<String>,
    pub province_code: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DistrictDetailQuery {
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VillageListRequest {
    pub district_code: Option<String>,
    pub regency_code: Option<String>,
    pub province_code: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct VillageDetailQuery {
    pub code: Option<String>,
    pub name: Option<String>,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_next: bool,
}

#[derive(Debug, Serialize)]
pub struct ProvinceListResponse {
    pub success: bool,
    pub message: String,
    pub data: Vec<ProvinceView>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct RegencyListResponse {
    pub success: bool,
    pub message: String,
    pub data: Vec<RegencyView>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct DistrictListResponse {
    pub success: bool,
    pub message: String,
    pub data: Vec<DistrictView>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct VillageListResponse {
    pub success: bool,
    pub message: String,
    pub data: Vec<VillageView>,
    pub pagination: PaginationInfo,
}

// Detail response DTOs
#[derive(Debug, Serialize)]
pub struct ProvinceDetailResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<ProvinceView>,
}

#[derive(Debug, Serialize)]
pub struct RegencyDetailResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<RegencyView>,
}

#[derive(Debug, Serialize)]
pub struct DistrictDetailResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<DistrictView>,
}

#[derive(Debug, Serialize)]
pub struct VillageDetailResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<VillageView>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, json, to_string};

    #[test]
    fn regions_province_list_request_fields() {
        let req = ProvinceListRequest {
            search: Some("jawa".to_string()),
            limit: Some(25),
            offset: Some(50),
        };
        assert_eq!(req.search.as_deref(), Some("jawa"));
        assert_eq!(req.limit, Some(25));
        assert_eq!(req.offset, Some(50));
    }

    #[test]
    fn regions_pagination_info_has_next_cases() {
        let mut p = PaginationInfo {
            total: 100,
            limit: 10,
            offset: 80,
            has_next: false,
        };
        p.has_next = (p.offset + p.limit) < p.total;
        assert!(p.has_next);

        p.offset = 90;
        p.has_next = (p.offset + p.limit) < p.total;
        assert!(!p.has_next);
    }

    #[test]
    fn regions_detail_query_construct() {
        let prov = ProvinceDetailQuery {
            code: Some("31".to_string()),
            name: None,
        };
        let reg = RegencyDetailQuery {
            code: Some("3174".to_string()),
            name: None,
        };
        let dis = DistrictDetailQuery {
            code: Some("317401".to_string()),
            name: None,
        };
        let vil = VillageDetailQuery {
            code: Some("31740101".to_string()),
            name: None,
        };

        assert_eq!(prov.code.as_deref(), Some("31"));
        assert_eq!(reg.code.as_deref(), Some("3174"));
        assert_eq!(dis.code.as_deref(), Some("317401"));
        assert_eq!(vil.code.as_deref(), Some("31740101"));
    }

    #[test]
    fn regions_province_list_response_serde() {
        let data = vec![ProvinceView {
            code: "31".to_string(),
            name: "DKI Jakarta".to_string(),
        }];
        let pagination = PaginationInfo {
            total: 1,
            limit: 10,
            offset: 0,
            has_next: false,
        };
        let resp = ProvinceListResponse {
            success: true,
            message: "ok".to_string(),
            data,
            pagination,
        };
        let s = to_string(&resp).expect("serialize response");
        let v: serde_json::Value = from_str(&s).expect("deserialize to value");
        assert_eq!(v["success"], json!(true));
        assert_eq!(v["data"][0]["code"], json!("31"));
        assert_eq!(v["pagination"]["total"], json!(1));
    }
}
