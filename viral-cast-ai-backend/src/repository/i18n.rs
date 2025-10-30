use serde_json::{json, Map, Value};
use sqlx::{postgres::PgRow, PgPool, Row};
use std::borrow::Cow;
use uuid::Uuid;

use crate::dto::i18n::{I18nEntity, UpsertTranslationSchema};

fn table_and_fk(entity: &I18nEntity) -> (&'static str, &'static str) {
    match entity {
        I18nEntity::IngredientCatalog => {
            ("ingredient_catalog_translations", "ingredient_catalog_uuid")
        }
        I18nEntity::Product => ("product_translations", "product_uuid"),
        I18nEntity::Category => ("category_translations", "category_uuid"),
        I18nEntity::RecipeSet => ("recipe_set_translations", "recipe_set_uuid"),
        I18nEntity::RecipeItem => ("recipe_item_translations", "recipe_item_uuid"),
        I18nEntity::IngredientStockMove => (
            "ingredient_stock_move_translations",
            "ingredient_stock_move_uuid",
        ),
        I18nEntity::IngredientStock => ("ingredient_stock_translations", "ingredient_stock_uuid"),
        I18nEntity::IngredientMarketPrice => (
            "ingredient_market_price_translations",
            "ingredient_market_price_uuid",
        ),
    }
}

fn allowed_columns(entity: &I18nEntity) -> Vec<&'static str> {
    match entity {
        I18nEntity::IngredientCatalog => vec!["name"],
        I18nEntity::Product => vec!["name", "description"],
        I18nEntity::Category => vec!["name"],
        I18nEntity::RecipeSet => vec!["name", "description"],
        I18nEntity::RecipeItem => vec!["name", "instructions", "notes"],
        I18nEntity::IngredientStockMove => vec!["name"],
        I18nEntity::IngredientStock => vec!["ingredient_name"],
        I18nEntity::IngredientMarketPrice => vec!["name"],
    }
}

pub async fn get_translation(
    pool: &PgPool,
    entity: &I18nEntity,
    uuid: Uuid,
    locale: &str,
) -> Result<Option<Value>, sqlx::Error> {
    let (table, fk_col) = table_and_fk(entity);
    let cols = allowed_columns(entity);

    let mut select_cols = String::from("uuid, ");
    select_cols.push_str("locale, translation_status");
    if !cols.is_empty() {
        select_cols.push_str(", ");
        select_cols.push_str(&cols.join(", "));
    }

    let sql = format!(
        "SELECT {select_cols} FROM {table} WHERE {fk_col} = $1 AND LOWER(locale) = LOWER($2) AND COALESCE(deleted_at,0)=0 LIMIT 1",
        select_cols = select_cols,
        table = table,
        fk_col = fk_col
    );

    let row_opt: Option<PgRow> = match sqlx::query(&sql)
        .bind(uuid)
        .bind(locale)
        .fetch_optional(pool)
        .await
    {
        Ok(row) => row,
        Err(sqlx::Error::Database(db_err)) if db_err.code() == Some(Cow::Borrowed("42P01")) => {
            // Translation table belum dimigrasi; perlakukan seperti tidak ada terjemahan
            return Ok(None);
        }
        Err(e) => return Err(e),
    };

    if let Some(row) = row_opt {
        let mut fields = Map::new();
        for &col in cols.iter() {
            let val: Option<String> = row.try_get(col).unwrap_or(None);
            fields.insert(col.to_string(), json!(val));
        }
        let res = json!({
            "uuid": row.get::<Uuid, _>("uuid"),
            "locale": row.get::<String, _>("locale"),
            "translation_status": row.get::<String, _>("translation_status"),
            "fields": Value::Object(fields),
        });
        Ok(Some(res))
    } else {
        Ok(None)
    }
}

pub async fn upsert_translation(
    pool: &PgPool,
    payload: &UpsertTranslationSchema,
) -> Result<Value, sqlx::Error> {
    let (table, fk_col) = table_and_fk(&payload.entity);
    let cols = allowed_columns(&payload.entity);

    // Determine provided fields
    let mut field_names: Vec<&str> = Vec::new();
    let mut field_values: Vec<Option<String>> = Vec::new();

    for &col in cols.iter() {
        match col {
            "name" => {
                field_names.push("name");
                field_values.push(payload.name.clone());
            }
            "description" => {
                field_names.push("description");
                field_values.push(payload.description.clone());
            }
            "instructions" => {
                field_names.push("instructions");
                field_values.push(payload.instructions.clone());
            }
            "notes" => {
                field_names.push("notes");
                field_values.push(payload.notes.clone());
            }
            "ingredient_name" => {
                field_names.push("ingredient_name");
                field_values.push(payload.ingredient_name.clone());
            }
            _ => {}
        }
    }

    // Check existing
    let existing_sql = format!(
        "SELECT uuid FROM {table} WHERE {fk_col} = $1 AND LOWER(locale) = LOWER($2) AND COALESCE(deleted_at,0)=0 LIMIT 1",
        table = table,
        fk_col = fk_col
    );
    let existing: Option<(Uuid,)> = sqlx::query_as(&existing_sql)
        .bind(payload.uuid)
        .bind(&payload.locale)
        .fetch_optional(pool)
        .await?;

    let status_upper = payload
        .translation_status
        .as_ref()
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "PENDING".to_string());

    if let Some((existing_id,)) = existing {
        // Build UPDATE statement
        let mut set_parts: Vec<String> = Vec::new();
        let mut binds: Vec<Option<String>> = Vec::new();

        for (i, name) in field_names.iter().enumerate() {
            // only set if provided
            if field_values[i].is_some() {
                set_parts.push(format!("{name} = ${}", i + 2));
                binds.push(field_values[i].clone());
            }
        }
        // Always update updated_at
        set_parts.push("updated_at = EXTRACT(EPOCH FROM now())::bigint".to_string());
        // Optionally update translation_status
        let mut sql = String::new();
        if !binds.is_empty() {
            sql.push_str(&format!(
                "UPDATE {table} SET {set_clause}",
                table = table,
                set_clause = set_parts.join(", ")
            ));
        } else {
            // No fields provided, only update status/updated_at
            sql.push_str(&format!(
                "UPDATE {table} SET updated_at = EXTRACT(EPOCH FROM now())::bigint",
                table = table
            ));
        }
        // Add translation_status
        sql.push_str(&format!(", translation_status = ${}", binds.len() + 2));
        // Where
        sql.push_str(" WHERE uuid = $1 RETURNING uuid, locale, translation_status");

        let mut query = sqlx::query(&sql).bind(existing_id);
        for b in binds {
            query = query.bind(b);
        }
        query = query.bind(status_upper);

        let row = query.fetch_one(pool).await?;
        let result = json!({
            "uuid": row.get::<Uuid, _>("uuid"),
            "locale": payload.locale.to_lowercase(),
            "translation_status": row.get::<String, _>("translation_status"),
            "fields": {},
        });
        Ok(result)
    } else {
        // Build INSERT statement
        let mut columns: Vec<&str> = vec![fk_col, "locale"];
        let mut placeholders: Vec<String> = vec!["$1".to_string(), "LOWER($2)".to_string()];
        let mut binds: Vec<Option<String>> = Vec::new();

        for (i, name) in field_names.iter().enumerate() {
            if field_values[i].is_some() {
                columns.push(name);
                placeholders.push(format!("${}", placeholders.len() + 1));
                binds.push(field_values[i].clone());
            }
        }
        columns.push("translation_status");
        placeholders.push(format!("${}", placeholders.len() + 1));

        let select_cols = {
            let mut s = String::from("uuid, locale, translation_status");
            if !cols.is_empty() {
                s.push_str(", ");
                s.push_str(&cols.join(", "));
            }
            s
        };

        let sql = format!(
            "INSERT INTO {table} ({cols}, created_at) VALUES ({phs}, EXTRACT(EPOCH FROM now())::bigint) RETURNING {select_cols}",
            table = table,
            cols = columns.join(", "),
            phs = placeholders.join(", "),
            select_cols = select_cols,
        );

        let mut query = sqlx::query(&sql).bind(payload.uuid).bind(&payload.locale);
        for b in binds {
            query = query.bind(b);
        }
        query = query.bind(status_upper);

        let row = query.fetch_one(pool).await?;
        // Build fields map
        let mut fields = Map::new();
        for &col in cols.iter() {
            let val: Option<String> = row.try_get(col).unwrap_or(None);
            fields.insert(col.to_string(), json!(val));
        }
        let result = json!({
            "uuid": row.get::<Uuid, _>("uuid"),
            "locale": row.get::<String, _>("locale"),
            "translation_status": row.get::<String, _>("translation_status"),
            "fields": Value::Object(fields),
        });
        Ok(result)
    }
}
