use crate::dto::stores::{CreateStoreSchema, GetStoreSchema, ProcessedStore, UpdateStoreSchema};
use chrono::Utc;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub async fn check_user_has_store(
    db: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "SELECT store_uuid FROM profiles WHERE user_uuid = $1 AND store_uuid IS NOT NULL AND deleted_at = 0",
    )
    .bind(user_uuid)
    .fetch_optional(db)
    .await?;

    Ok(result.is_some())
}

pub async fn update_user_profile_store(
    db: &Pool<Postgres>,
    user_uuid: Uuid,
    store_uuid: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE profiles SET store_uuid = $1, updated_at = $2 WHERE user_uuid = $3",
        store_uuid,
        chrono::Utc::now().timestamp(),
        user_uuid
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn create_store(
    db: &Pool<Postgres>,
    body: CreateStoreSchema,
) -> Result<ProcessedStore, sqlx::Error> {
    let name = body.name.unwrap_or_default();

    let query = r#"
        INSERT INTO stores (
            name, brand_url, province_code, regency_code, district_code, village_code,
            rt, rw, postal_code, telp, whatsapp, instagram
        ) VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9, $10, $11, $12
        ) RETURNING uuid, name, brand_url, province_code, regency_code, district_code, village_code,
                  rt, rw, postal_code, telp, whatsapp, instagram,
                  opening_time, middle_closing_time, closing_time,
                  created_at, updated_at
    "#;

    let store = sqlx::query_as::<_, GetStoreSchema>(query)
        .bind(name)
        .bind(body.brand_url)
        .bind(body.province_code)
        .bind(body.regency_code)
        .bind(body.district_code)
        .bind(body.village_code)
        .bind(body.rt)
        .bind(body.rw)
        .bind(body.postal_code)
        .bind(body.telp)
        .bind(body.whatsapp)
        .bind(body.instagram)
        .fetch_one(db)
        .await?;

    Ok(ProcessedStore {
        uuid: store.uuid,
        name: store.name,
        brand_url: store.brand_url,
        province_code: store.province_code,
        regency_code: store.regency_code,
        district_code: store.district_code,
        village_code: store.village_code,
        rt: store.rt,
        rw: store.rw,
        postal_code: store.postal_code,
        telp: store.telp,
        whatsapp: store.whatsapp,
        instagram: store.instagram,
        opening_time: store.opening_time,
        middle_closing_time: store.middle_closing_time,
        closing_time: store.closing_time,
        created_at: store.created_at,
        updated_at: store.updated_at,
    })
}

pub async fn get_store_by_uuid(
    db: &Pool<Postgres>,
    store_uuid: Uuid,
) -> Result<Option<ProcessedStore>, sqlx::Error> {
    let query = r#"
        SELECT uuid, name, brand_url, province_code, regency_code, district_code, village_code,
               rt, rw, postal_code, telp, whatsapp, instagram,
               opening_time, middle_closing_time, closing_time,
               created_at, updated_at
        FROM stores WHERE uuid = $1 AND deleted_at = 0
    "#;

    let row = sqlx::query_as::<_, GetStoreSchema>(query)
        .bind(store_uuid)
        .fetch_optional(db)
        .await?;

    Ok(row.map(|store| ProcessedStore {
        uuid: store.uuid,
        name: store.name,
        brand_url: store.brand_url,
        province_code: store.province_code,
        regency_code: store.regency_code,
        district_code: store.district_code,
        village_code: store.village_code,
        rt: store.rt,
        rw: store.rw,
        postal_code: store.postal_code,
        telp: store.telp,
        whatsapp: store.whatsapp,
        instagram: store.instagram,
        opening_time: store.opening_time,
        middle_closing_time: store.middle_closing_time,
        closing_time: store.closing_time,
        created_at: store.created_at,
        updated_at: store.updated_at,
    }))
}

pub async fn get_store_by_user_uuid(
    db: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Option<ProcessedStore>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.uuid,
            s.name,
            s.brand_url,
            s.province_code,
            s.regency_code,
            s.district_code,
            s.village_code,
            s.rt,
            s.rw,
            s.postal_code,
            s.telp,
            s.whatsapp,
            s.instagram,
            s.opening_time,
            s.middle_closing_time,
            s.closing_time,
            s.created_at,
            s.updated_at
        FROM profiles p
        JOIN stores s ON p.store_uuid = s.uuid
        WHERE p.user_uuid = $1
          AND p.store_uuid IS NOT NULL
          AND COALESCE(p.deleted_at, 0) = 0
          AND s.deleted_at = 0
        ORDER BY s.updated_at DESC
        LIMIT 1
    "#;

    let row = sqlx::query_as::<_, GetStoreSchema>(query)
        .bind(user_uuid)
        .fetch_optional(db)
        .await?;

    Ok(row.map(|store| ProcessedStore {
        uuid: store.uuid,
        name: store.name,
        brand_url: store.brand_url,
        province_code: store.province_code,
        regency_code: store.regency_code,
        district_code: store.district_code,
        village_code: store.village_code,
        rt: store.rt,
        rw: store.rw,
        postal_code: store.postal_code,
        telp: store.telp,
        whatsapp: store.whatsapp,
        instagram: store.instagram,
        opening_time: store.opening_time,
        middle_closing_time: store.middle_closing_time,
        closing_time: store.closing_time,
        created_at: store.created_at,
        updated_at: store.updated_at,
    }))
}

pub async fn update_store(
    db: &Pool<Postgres>,
    store_uuid: Uuid,
    body: UpdateStoreSchema,
) -> Result<ProcessedStore, sqlx::Error> {
    // Fetch current store
    let current = sqlx::query_as::<_, GetStoreSchema>(
        r#"SELECT uuid, name, brand_url, province_code, regency_code, district_code, village_code,
            rt, rw, postal_code, telp, whatsapp, instagram,
            opening_time, middle_closing_time, closing_time,
            created_at, updated_at
           FROM stores WHERE uuid = $1 AND deleted_at = 0"#,
    )
    .bind(store_uuid)
    .fetch_one(db)
    .await?;

    let name = body.name.unwrap_or(current.name);
    // Gunakan Unix timestamp (ms) langsung; jika tidak dikirim, pertahankan nilai lama
    let opening_time: Option<i64> = match &body.opening_time {
        Some(v) if *v > 0 => Some(*v),
        Some(_) => None,
        None => current.opening_time,
    };
    let middle_closing_time: Option<i64> = match &body.middle_closing_time {
        Some(v) if *v > 0 => Some(*v),
        Some(_) => None,
        None => current.middle_closing_time,
    };
    let closing_time: Option<i64> = match &body.closing_time {
        Some(v) if *v > 0 => Some(*v),
        Some(_) => None,
        None => current.closing_time,
    };
    let query = r#"
        UPDATE stores SET
            name = $1,
            brand_url = $2,
            province_code = $3,
            regency_code = $4,
            district_code = $5,
            village_code = $6,
            rt = $7,
            rw = $8,
            postal_code = $9,
            telp = $10,
            whatsapp = $11,
            instagram = $12,
            opening_time = $13,
            middle_closing_time = $14,
            closing_time = $15,
            updated_at = (EXTRACT(EPOCH FROM NOW()) * 1000)::bigint
        WHERE uuid = $16 AND deleted_at = 0
        RETURNING uuid, name, brand_url, province_code, regency_code, district_code, village_code,
                  rt, rw, postal_code, telp, whatsapp, instagram,
                  opening_time, middle_closing_time, closing_time,
                  created_at, updated_at
    "#;

    let updated = sqlx::query_as::<_, GetStoreSchema>(query)
        .bind(name)
        .bind(body.brand_url.or(current.brand_url))
        .bind(body.province_code.or(current.province_code))
        .bind(body.regency_code.or(current.regency_code))
        .bind(body.district_code.or(current.district_code))
        .bind(body.village_code.or(current.village_code))
        .bind(body.rt.or(current.rt))
        .bind(body.rw.or(current.rw))
        .bind(body.postal_code.or(current.postal_code))
        .bind(body.telp.or(current.telp))
        .bind(body.whatsapp.or(current.whatsapp))
        .bind(body.instagram.or(current.instagram))
        .bind(opening_time)
        .bind(middle_closing_time)
        .bind(closing_time)
        .bind(store_uuid)
        .fetch_one(db)
        .await?;

    Ok(ProcessedStore {
        uuid: updated.uuid,
        name: updated.name,
        brand_url: updated.brand_url,
        province_code: updated.province_code,
        regency_code: updated.regency_code,
        district_code: updated.district_code,
        village_code: updated.village_code,
        rt: updated.rt,
        rw: updated.rw,
        postal_code: updated.postal_code,
        telp: updated.telp,
        whatsapp: updated.whatsapp,
        instagram: updated.instagram,
        opening_time: updated.opening_time,
        middle_closing_time: updated.middle_closing_time,
        closing_time: updated.closing_time,
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    })
}
