use crate::dto::regions::{
    DistrictDetailQuery, DistrictListRequest, ProvinceDetailQuery, ProvinceListRequest,
    RegencyDetailQuery, RegencyListRequest, VillageDetailQuery, VillageListRequest,
};
use crate::models::regions::{DistrictView, ProvinceView, RegencyView, VillageView};
use sqlx::{Pool, Postgres};

// Provinces
pub async fn list_provinces(
    db: &Pool<Postgres>,
    params: &ProvinceListRequest,
) -> Result<(Vec<ProvinceView>, i64, i64, i64), sqlx::Error> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let from_clause = "FROM province".to_string();
    let mut conditions: Vec<String> = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();

    conditions.push("deleted_at = 0".to_string());

    if let Some(search) = &params.search {
        let s = search.trim();
        if !s.is_empty() {
            conditions.push(format!("name ILIKE ${}", bind_values.len() + 1));
            bind_values.push(format!("%{}%", s));
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let mut data_query = format!(
        "SELECT code, name {}{} ORDER BY name LIMIT {} OFFSET {}",
        from_clause, where_clause, limit, offset
    );
    let count_query = format!("SELECT COUNT(*) {}{}", from_clause, where_clause);

    let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
    for v in &bind_values {
        count_builder = count_builder.bind(v);
    }
    let total = count_builder.fetch_one(db).await?;

    let mut data_builder = sqlx::query_as::<_, ProvinceView>(&data_query);
    for v in &bind_values {
        data_builder = data_builder.bind(v);
    }
    let rows = data_builder.fetch_all(db).await?;

    Ok((rows, total, limit, offset))
}

pub async fn get_province_by_code(
    db: &Pool<Postgres>,
    code: &str,
) -> Result<Option<ProvinceView>, sqlx::Error> {
    let q = "SELECT code, name FROM province WHERE code = $1 AND deleted_at = 0";
    sqlx::query_as::<_, ProvinceView>(q)
        .bind(code)
        .fetch_optional(db)
        .await
}

pub async fn get_province_detail(
    db: &Pool<Postgres>,
    params: &ProvinceDetailQuery,
) -> Result<Option<ProvinceView>, sqlx::Error> {
    if let Some(code) = params.code.clone() {
        let q = r#"SELECT code, name FROM province WHERE code = $1 AND deleted_at = 0"#;
        return sqlx::query_as::<_, ProvinceView>(q)
            .bind(code)
            .fetch_optional(db)
            .await;
    }
    if let Some(name) = params.name.clone() {
        let q = r#"SELECT code, name FROM province WHERE name ILIKE $1 AND deleted_at = 0 LIMIT 1"#;
        let pat = format!("%{}%", name);
        return sqlx::query_as::<_, ProvinceView>(q)
            .bind(pat)
            .fetch_optional(db)
            .await;
    }
    Ok(None)
}

// Regencies
pub async fn list_regencies(
    db: &Pool<Postgres>,
    params: &RegencyListRequest,
) -> Result<(Vec<RegencyView>, i64, i64, i64), sqlx::Error> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let from_clause = r#"
        FROM regency r
        JOIN province p ON r.province_uuid = p.uuid
    "#
    .to_string();
    let mut conditions: Vec<String> = vec![
        "r.deleted_at = 0".to_string(),
        "p.deleted_at = 0".to_string(),
    ];
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(province_code) = &params.province_code {
        let pc = province_code.trim();
        if !pc.is_empty() {
            conditions.push(format!("p.code = ${}", bind_values.len() + 1));
            bind_values.push(pc.to_string());
        }
    }
    if let Some(search) = &params.search {
        let s = search.trim();
        if !s.is_empty() {
            conditions.push(format!("r.name ILIKE ${}", bind_values.len() + 1));
            bind_values.push(format!("%{}%", s));
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let mut data_query = format!("SELECT r.code, r.name, p.code as province_code, p.name as province_name {}{} ORDER BY r.name LIMIT {} OFFSET {}", from_clause, where_clause, limit, offset);
    let count_query = format!("SELECT COUNT(*) {}{}", from_clause, where_clause);

    let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
    for v in &bind_values {
        count_builder = count_builder.bind(v);
    }
    let total = count_builder.fetch_one(db).await?;

    let mut data_builder = sqlx::query_as::<_, RegencyView>(&data_query);
    for v in &bind_values {
        data_builder = data_builder.bind(v);
    }
    let rows = data_builder.fetch_all(db).await?;

    Ok((rows, total, limit, offset))
}

pub async fn get_regency_by_code(
    db: &Pool<Postgres>,
    code: &str,
) -> Result<Option<RegencyView>, sqlx::Error> {
    let q = r#"
        SELECT r.code, r.name, p.code as province_code, p.name as province_name
        FROM regency r
        JOIN province p ON r.province_uuid = p.uuid
        WHERE r.code = $1 AND r.deleted_at = 0 AND p.deleted_at = 0
    "#;
    sqlx::query_as::<_, RegencyView>(q)
        .bind(code)
        .fetch_optional(db)
        .await
}

pub async fn get_regency_detail(
    db: &Pool<Postgres>,
    params: &RegencyDetailQuery,
) -> Result<Option<RegencyView>, sqlx::Error> {
    if let Some(code) = params.code.clone().and_then(|c| {
        let t = c.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    }) {
        let q = r#"
            SELECT r.code, r.name, p.code as province_code, p.name as province_name
            FROM regency r
            JOIN province p ON r.province_uuid = p.uuid
            WHERE r.code = $1 AND r.deleted_at = 0 AND p.deleted_at = 0
        "#;
        return sqlx::query_as::<_, RegencyView>(q)
            .bind(code)
            .fetch_optional(db)
            .await;
    }
    if let Some(name) = params.name.clone().and_then(|n| {
        let t = n.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    }) {
        let q = r#"
            SELECT r.code, r.name, p.code as province_code, p.name as province_name
            FROM regency r
            JOIN province p ON r.province_uuid = p.uuid
            WHERE r.name ILIKE $1 AND r.deleted_at = 0 AND p.deleted_at = 0
        "#;
        let pat = format!("%{}%", name);
        return sqlx::query_as::<_, RegencyView>(q)
            .bind(pat)
            .fetch_optional(db)
            .await;
    }
    Ok(None)
}

// Districts
pub async fn list_districts(
    db: &Pool<Postgres>,
    params: &DistrictListRequest,
) -> Result<(Vec<DistrictView>, i64, i64, i64), sqlx::Error> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let from_clause = r#"
        FROM district d
        JOIN regency r ON d.regency_uuid = r.uuid
        JOIN province p ON r.province_uuid = p.uuid
    "#
    .to_string();
    let mut conditions: Vec<String> = vec![
        "d.deleted_at = 0".to_string(),
        "r.deleted_at = 0".to_string(),
        "p.deleted_at = 0".to_string(),
    ];
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(regency_code) = &params.regency_code {
        conditions.push(format!("r.code = ${}", bind_values.len() + 1));
        bind_values.push(regency_code.to_string());
    }
    if let Some(province_code) = &params.province_code {
        conditions.push(format!("p.code = ${}", bind_values.len() + 1));
        bind_values.push(province_code.to_string());
    }
    if let Some(search) = &params.search {
        let s = search.trim();
        if !s.is_empty() {
            conditions.push(format!("d.name ILIKE ${}", bind_values.len() + 1));
            bind_values.push(format!("%{}%", s));
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let mut data_query = format!("SELECT d.code, d.name, r.code as regency_code, r.name as regency_name, p.code as province_code, p.name as province_name {}{} ORDER BY d.name LIMIT {} OFFSET {}", from_clause, where_clause, limit, offset);
    let count_query = format!("SELECT COUNT(*) {}{}", from_clause, where_clause);

    let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
    for v in &bind_values {
        count_builder = count_builder.bind(v);
    }
    let total = count_builder.fetch_one(db).await?;

    let mut data_builder = sqlx::query_as::<_, DistrictView>(&data_query);
    for v in &bind_values {
        data_builder = data_builder.bind(v);
    }
    let rows = data_builder.fetch_all(db).await?;

    Ok((rows, total, limit, offset))
}

pub async fn get_district_by_code(
    db: &Pool<Postgres>,
    code: &str,
) -> Result<Option<DistrictView>, sqlx::Error> {
    let q = r#"
        SELECT d.code, d.name,
               r.code as regency_code, r.name as regency_name,
               p.code as province_code, p.name as province_name
        FROM district d
        JOIN regency r ON d.regency_uuid = r.uuid
        JOIN province p ON r.province_uuid = p.uuid
        WHERE d.code = $1 AND d.deleted_at = 0 AND r.deleted_at = 0 AND p.deleted_at = 0
    "#;
    sqlx::query_as::<_, DistrictView>(q)
        .bind(code)
        .fetch_optional(db)
        .await
}

pub async fn get_district_detail(
    db: &Pool<Postgres>,
    params: &DistrictDetailQuery,
) -> Result<Option<DistrictView>, sqlx::Error> {
    if let Some(code) = params.code.clone().and_then(|c| {
        let t = c.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    }) {
        let q = r#"
            SELECT d.code, d.name,
                   r.code as regency_code, r.name as regency_name,
                   p.code as province_code, p.name as province_name
            FROM district d
            JOIN regency r ON d.regency_uuid = r.uuid
            JOIN province p ON r.province_uuid = p.uuid
            WHERE d.code = $1 AND d.deleted_at = 0 AND r.deleted_at = 0 AND p.deleted_at = 0
        "#;
        return sqlx::query_as::<_, DistrictView>(q)
            .bind(code)
            .fetch_optional(db)
            .await;
    }
    if let Some(name) = params.name.clone().and_then(|n| {
        let t = n.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    }) {
        let q = r#"
            SELECT d.code, d.name,
                   r.code as regency_code, r.name as regency_name,
                   p.code as province_code, p.name as province_name
            FROM district d
            JOIN regency r ON d.regency_uuid = r.uuid
            JOIN province p ON r.province_uuid = p.uuid
            WHERE d.name ILIKE $1 AND d.deleted_at = 0 AND r.deleted_at = 0 AND p.deleted_at = 0
        "#;
        let pat = format!("%{}%", name);
        return sqlx::query_as::<_, DistrictView>(q)
            .bind(pat)
            .fetch_optional(db)
            .await;
    }
    Ok(None)
}

// Villages
pub async fn list_villages(
    db: &Pool<Postgres>,
    params: &VillageListRequest,
) -> Result<(Vec<VillageView>, i64, i64, i64), sqlx::Error> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let from_clause = r#"
        FROM village v
        JOIN district d ON v.district_uuid = d.uuid
        JOIN regency r ON d.regency_uuid = r.uuid
        JOIN province p ON r.province_uuid = p.uuid
    "#
    .to_string();
    let mut conditions: Vec<String> = vec![
        "v.deleted_at = 0".to_string(),
        "d.deleted_at = 0".to_string(),
        "r.deleted_at = 0".to_string(),
        "p.deleted_at = 0".to_string(),
    ];
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(district_code) = &params.district_code {
        conditions.push(format!("d.code = ${}", bind_values.len() + 1));
        bind_values.push(district_code.to_string());
    }
    if let Some(regency_code) = &params.regency_code {
        conditions.push(format!("r.code = ${}", bind_values.len() + 1));
        bind_values.push(regency_code.to_string());
    }
    if let Some(province_code) = &params.province_code {
        conditions.push(format!("p.code = ${}", bind_values.len() + 1));
        bind_values.push(province_code.to_string());
    }
    if let Some(search) = &params.search {
        let s = search.trim();
        if !s.is_empty() {
            conditions.push(format!("v.name ILIKE ${}", bind_values.len() + 1));
            bind_values.push(format!("%{}%", s));
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let mut data_query = format!("SELECT v.code, v.name, d.code as district_code, d.name as district_name, r.code as regency_code, r.name as regency_name, p.code as province_code, p.name as province_name {}{} ORDER BY v.name LIMIT {} OFFSET {}", from_clause, where_clause, limit, offset);
    let count_query = format!("SELECT COUNT(*) {}{}", from_clause, where_clause);

    let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
    for v in &bind_values {
        count_builder = count_builder.bind(v);
    }
    let total = count_builder.fetch_one(db).await?;

    let mut data_builder = sqlx::query_as::<_, VillageView>(&data_query);
    for v in &bind_values {
        data_builder = data_builder.bind(v);
    }
    let rows = data_builder.fetch_all(db).await?;

    Ok((rows, total, limit, offset))
}

pub async fn get_village_by_code(
    db: &Pool<Postgres>,
    code: &str,
) -> Result<Option<VillageView>, sqlx::Error> {
    let q = r#"
        SELECT v.code, v.name,
               d.code as district_code, d.name as district_name,
               r.code as regency_code, r.name as regency_name,
               p.code as province_code, p.name as province_name
        FROM village v
        JOIN district d ON v.district_uuid = d.uuid
        JOIN regency r ON d.regency_uuid = r.uuid
        JOIN province p ON r.province_uuid = p.uuid
        WHERE v.code = $1 AND v.deleted_at = 0 AND d.deleted_at = 0 AND r.deleted_at = 0 AND p.deleted_at = 0
    "#;
    sqlx::query_as::<_, VillageView>(q)
        .bind(code)
        .fetch_optional(db)
        .await
}

pub async fn get_village_detail(
    db: &Pool<Postgres>,
    params: &VillageDetailQuery,
) -> Result<Option<VillageView>, sqlx::Error> {
    if let Some(code) = params.code.clone().and_then(|c| {
        let t = c.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    }) {
        let q = r#"
            SELECT v.code, v.name,
                   d.code as district_code, d.name as district_name,
                   r.code as regency_code, r.name as regency_name,
                   p.code as province_code, p.name as province_name
            FROM village v
            JOIN district d ON v.district_uuid = d.uuid
            JOIN regency r ON d.regency_uuid = r.uuid
            JOIN province p ON r.province_uuid = p.uuid
            WHERE v.code = $1 AND v.deleted_at = 0 AND d.deleted_at = 0 AND r.deleted_at = 0 AND p.deleted_at = 0
        "#;
        return sqlx::query_as::<_, VillageView>(q)
            .bind(code)
            .fetch_optional(db)
            .await;
    }
    if let Some(name) = params.name.clone().and_then(|n| {
        let t = n.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    }) {
        let q = r#"
            SELECT v.code, v.name,
                   d.code as district_code, d.name as district_name,
                   r.code as regency_code, r.name as regency_name,
                   p.code as province_code, p.name as province_name
            FROM village v
            JOIN district d ON v.district_uuid = d.uuid
            JOIN regency r ON d.regency_uuid = r.uuid
            JOIN province p ON r.province_uuid = p.uuid
            WHERE v.name ILIKE $1 AND v.deleted_at = 0 AND d.deleted_at = 0 AND r.deleted_at = 0 AND p.deleted_at = 0
        "#;
        let pat = format!("%{}%", name);
        return sqlx::query_as::<_, VillageView>(q)
            .bind(pat)
            .fetch_optional(db)
            .await;
    }
    Ok(None)
}
