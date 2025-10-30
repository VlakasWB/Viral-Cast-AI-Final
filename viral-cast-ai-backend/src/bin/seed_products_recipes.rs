use anyhow::{Context, Result};
use chrono::Utc;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

#[derive(Clone, Copy)]
struct IngredientUsage {
    ingredient_name: &'static str,
    quantity: f64,
}

#[derive(Clone, Copy)]
struct ProductSeed {
    name: &'static str,
    image: &'static str,
    price_idr: f64,
    ingredients: &'static [IngredientUsage],
}

#[derive(Clone, Copy)]
struct CategorySeed {
    name: &'static str,
    products: &'static [ProductSeed],
}

const DESSERT_PRODUCTS: &[ProductSeed] = &[
    ProductSeed {
        name: "Bamboo Taro Dessert",
        image: "bamboo-taro-dessert.jpg",
        price_idr: 42000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Bamboo Taro Dessert Base",
                quantity: 0.30,
            },
            IngredientUsage {
                ingredient_name: "Susu Segar Pasteurisasi",
                quantity: 0.20,
            },
        ],
    },
    ProductSeed {
        name: "Chocolate Berry Mousse Cake",
        image: "chocolate-berry-mousse-cake.jpg",
        price_idr: 68000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Chocolate Berry Mousse Layer",
                quantity: 0.40,
            },
            IngredientUsage {
                ingredient_name: "Raspberry Cream Filling",
                quantity: 0.25,
            },
        ],
    },
    ProductSeed {
        name: "Elegant White Wedding Cake",
        image: "elegant-white-wedding-cake.jpg",
        price_idr: 115000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Wedding Cake Fondant Sheet",
                quantity: 0.50,
            },
            IngredientUsage {
                ingredient_name: "Cake Mix Vanilla Premium",
                quantity: 0.60,
            },
        ],
    },
    ProductSeed {
        name: "Pumpkin Dessert Flatlay",
        image: "pumpkin-dessert-flatlay.jpg",
        price_idr: 52000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Pumpkin Spice Puree",
                quantity: 0.30,
            },
            IngredientUsage {
                ingredient_name: "Kulit Pie Beku",
                quantity: 0.40,
            },
        ],
    },
    ProductSeed {
        name: "Raspberry Cream Cake Slice",
        image: "raspberry-cream-cake-slice.jpg",
        price_idr: 35000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Cake Mix Vanilla Premium",
                quantity: 0.20,
            },
            IngredientUsage {
                ingredient_name: "Raspberry Cream Filling",
                quantity: 0.15,
            },
        ],
    },
];

const BEVERAGE_PRODUCTS: &[ProductSeed] = &[
    ProductSeed {
        name: "Cappuccino and Truffles",
        image: "cappuccino-and-truffles.jpg",
        price_idr: 39000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Cappuccino Praline Concentrate",
                quantity: 0.30,
            },
            IngredientUsage {
                ingredient_name: "Biji Kopi Arabika Medium Roast",
                quantity: 0.04,
            },
        ],
    },
    ProductSeed {
        name: "Citrus Bar Cooler",
        image: "citrus-bar-cooler.jpg",
        price_idr: 32000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Citrus Cooler Syrup",
                quantity: 0.20,
            },
            IngredientUsage {
                ingredient_name: "Orange Sparkler Nectar",
                quantity: 0.20,
            },
        ],
    },
    ProductSeed {
        name: "Deluxe Caramel Milkshake",
        image: "deluxe-caramel-milkshake.jpg",
        price_idr: 38000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Caramel Milkshake Mix",
                quantity: 0.35,
            },
            IngredientUsage {
                ingredient_name: "Barista Full Cream Milk",
                quantity: 0.30,
            },
        ],
    },
    ProductSeed {
        name: "Fizzy Cola Duo",
        image: "fizzy-cola-duo.jpg",
        price_idr: 27000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Craft Cola Syrup",
                quantity: 0.25,
            },
            IngredientUsage {
                ingredient_name: "Sirup Gula Cair",
                quantity: 0.10,
            },
        ],
    },
    ProductSeed {
        name: "Floral Black Coffee",
        image: "floral-black-coffee.jpg",
        price_idr: 30000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Floral Coffee Infusion",
                quantity: 0.25,
            },
            IngredientUsage {
                ingredient_name: "Biji Kopi Arabika Medium Roast",
                quantity: 0.05,
            },
        ],
    },
    ProductSeed {
        name: "Fresh Milk Pour",
        image: "fresh-milk-pour.jpg",
        price_idr: 22000.0,
        ingredients: &[IngredientUsage {
            ingredient_name: "Barista Full Cream Milk",
            quantity: 0.35,
        }],
    },
    ProductSeed {
        name: "Heart Latte Art Cup",
        image: "heart-latte-art-cup.jpg",
        price_idr: 33000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Latte Art Microfoam Mix",
                quantity: 0.20,
            },
            IngredientUsage {
                ingredient_name: "Barista Full Cream Milk",
                quantity: 0.25,
            },
        ],
    },
    ProductSeed {
        name: "Iced Matcha Latte Sprinkle",
        image: "iced-matcha-latte-sprinkle.jpg",
        price_idr: 36000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Matcha Creme Drizzle",
                quantity: 0.20,
            },
            IngredientUsage {
                ingredient_name: "Bubuk Matcha Premium",
                quantity: 0.05,
            },
        ],
    },
    ProductSeed {
        name: "Iced Milk Coffee To Go",
        image: "iced-milk-coffee-to-go.jpg",
        price_idr: 33000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Iced Milk Coffee Base",
                quantity: 0.30,
            },
            IngredientUsage {
                ingredient_name: "Krim Kental Manis Cair",
                quantity: 0.15,
            },
        ],
    },
    ProductSeed {
        name: "Iced Orange Sparkler",
        image: "iced-orange-sparkler.jpg",
        price_idr: 31000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Orange Sparkler Nectar",
                quantity: 0.25,
            },
            IngredientUsage {
                ingredient_name: "Sparkling Pink Lemonade Syrup",
                quantity: 0.10,
            },
        ],
    },
    ProductSeed {
        name: "Purple Sprinkle Frappe",
        image: "purple-sprinkle-frappe.jpg",
        price_idr: 41000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Lavender Ube Frappe Base",
                quantity: 0.28,
            },
            IngredientUsage {
                ingredient_name: "Krim Kental Manis Cair",
                quantity: 0.18,
            },
        ],
    },
    ProductSeed {
        name: "Sparkling Pink Lemonade",
        image: "sparkling-pink-lemonade.jpg",
        price_idr: 28000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Sparkling Pink Lemonade Syrup",
                quantity: 0.20,
            },
            IngredientUsage {
                ingredient_name: "Sirup Gula Cair",
                quantity: 0.15,
            },
        ],
    },
    ProductSeed {
        name: "Strawberry Matcha Jar Latte",
        image: "strawberry-matcha-jar-latte.jpg",
        price_idr: 39000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Strawberry Matcha Jam",
                quantity: 0.25,
            },
            IngredientUsage {
                ingredient_name: "Bubuk Matcha Premium",
                quantity: 0.05,
            },
        ],
    },
    ProductSeed {
        name: "Sunset Orange Iced Drink",
        image: "sunset-orange-iced-drink.jpg",
        price_idr: 30000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Sunset Orange Tea Base",
                quantity: 0.25,
            },
            IngredientUsage {
                ingredient_name: "Citrus Cooler Syrup",
                quantity: 0.15,
            },
        ],
    },
];

const SAVORY_PRODUCTS: &[ProductSeed] = &[
    ProductSeed {
        name: "Classic Sesame Burger",
        image: "classic-sesame-burger.jpg",
        price_idr: 55000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Sesame Burger Bun Dough",
                quantity: 1.0,
            },
            IngredientUsage {
                ingredient_name: "Patty Daging Sapi Burger",
                quantity: 0.35,
            },
        ],
    },
    ProductSeed {
        name: "Crispy Spring Roll Stack",
        image: "crispy-spring-roll-stack.jpg",
        price_idr: 48000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Spring Roll Vegetable Filling",
                quantity: 0.35,
            },
            IngredientUsage {
                ingredient_name: "Minyak Goreng Sawit Premium",
                quantity: 0.05,
            },
        ],
    },
    ProductSeed {
        name: "Garden Veggie Pizza",
        image: "garden-veggie-pizza.jpg",
        price_idr: 75000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Garden Veggie Pizza Topping",
                quantity: 0.25,
            },
            IngredientUsage {
                ingredient_name: "Adonan Pizza Beku",
                quantity: 0.5,
            },
            IngredientUsage {
                ingredient_name: "Keju Mozzarella Block",
                quantity: 0.15,
            },
        ],
    },
    ProductSeed {
        name: "Golden French Fries Bowl",
        image: "golden-french-fries-bowl.jpg",
        price_idr: 30000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Golden Fry Potatoes",
                quantity: 0.40,
            },
            IngredientUsage {
                ingredient_name: "Minyak Goreng Sawit Premium",
                quantity: 0.06,
            },
        ],
    },
    ProductSeed {
        name: "Rustic Margherita Pizza",
        image: "rustic-margherita-pizza.jpg",
        price_idr: 78000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Wood-Fired Pizza Sauce",
                quantity: 0.18,
            },
            IngredientUsage {
                ingredient_name: "Adonan Pizza Beku",
                quantity: 0.50,
            },
            IngredientUsage {
                ingredient_name: "Keju Mozzarella Block",
                quantity: 0.20,
            },
        ],
    },
];

const PLANT_BASED_PRODUCTS: &[ProductSeed] = &[
    ProductSeed {
        name: "Spicy Tofu Cubes",
        image: "spicy-tofu-cubes.jpg",
        price_idr: 52000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Chili Tofu Cubes",
                quantity: 0.30,
            },
            IngredientUsage {
                ingredient_name: "Tofu Rainbow Bowl Mix",
                quantity: 0.20,
            },
        ],
    },
    ProductSeed {
        name: "Tofu Rainbow Bowl",
        image: "tofu-rainbow-bowl.jpg",
        price_idr: 54000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Tofu Rainbow Bowl Mix",
                quantity: 0.25,
            },
            IngredientUsage {
                ingredient_name: "Sayuran Campur Beku",
                quantity: 0.20,
            },
        ],
    },
    ProductSeed {
        name: "Veggie Ramen Bowls",
        image: "veggie-ramen-bowls.jpg",
        price_idr: 60000.0,
        ingredients: &[
            IngredientUsage {
                ingredient_name: "Vegetable Ramen Broth",
                quantity: 0.40,
            },
            IngredientUsage {
                ingredient_name: "Bok Choy Segar",
                quantity: 0.15,
            },
            IngredientUsage {
                ingredient_name: "Paprika Merah Besar",
                quantity: 0.10,
            },
        ],
    },
];

const CATEGORY_SEEDS: &[CategorySeed] = &[
    CategorySeed {
        name: "Dessert",
        products: DESSERT_PRODUCTS,
    },
    CategorySeed {
        name: "Beverage",
        products: BEVERAGE_PRODUCTS,
    },
    CategorySeed {
        name: "Savory",
        products: SAVORY_PRODUCTS,
    },
    CategorySeed {
        name: "Plant-Based",
        products: PLANT_BASED_PRODUCTS,
    },
];

struct IngredientInfo {
    uuid: Uuid,
    default_price: Decimal,
}

fn deterministic_uuid(key: &str) -> Uuid {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0F) | 0x50;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    Uuid::from_bytes(bytes)
}

fn product_sku(name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let digest = hasher.finalize();
    let mut code = String::with_capacity(6);
    for byte in digest.iter().take(3) {
        code.push_str(&format!("{:02X}", byte));
    }
    format!("SKU{}", code)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("gagal terhubung ke database")?;

    seed_products_and_recipes(&pool).await?;
    println!("✅ seeding products, recipe_sets, and recipe_items selesai");
    Ok(())
}

async fn seed_products_and_recipes(pool: &PgPool) -> Result<()> {
    let now = Utc::now().timestamp_millis();
    let mut tx = pool.begin().await?;
    let ingredient_map = load_ingredient_map(&mut tx).await?;

    for category in CATEGORY_SEEDS {
        let category_uuid = ensure_category(&mut tx, category.name, now).await?;

        for product in category.products {
            let product_uuid = upsert_product(&mut tx, product, category_uuid, now).await?;
            let recipe_set_uuid = upsert_recipe_set(&mut tx, product, now).await?;
            link_product_to_recipe(&mut tx, product_uuid, recipe_set_uuid, now).await?;
            clear_recipe_items(&mut tx, recipe_set_uuid).await?;

            for usage in product.ingredients {
                let info = ingredient_map.get(usage.ingredient_name).with_context(|| {
                    format!(
                        "Ingredient '{}' tidak ditemukan di ingredient_catalog",
                        usage.ingredient_name
                    )
                })?;
                let quantity = Decimal::from_f64(usage.quantity)
                    .with_context(|| {
                        format!("gagal konversi quantity untuk {}", usage.ingredient_name)
                    })?
                    .round_dp(4);
                let price = info.default_price;
                let total_value = (quantity * price).round_dp(4);
                let move_uuid = deterministic_uuid(&format!(
                    "prod-move-{}-{}",
                    product.name, usage.ingredient_name
                ));
                let stock_uuid = deterministic_uuid(&format!(
                    "prod-stock-{}-{}",
                    product.name, usage.ingredient_name
                ));

                insert_production_move(
                    &mut tx,
                    move_uuid,
                    info.uuid,
                    quantity,
                    price,
                    now,
                    product_uuid,
                    usage.ingredient_name,
                )
                .await?;

                insert_production_stock(
                    &mut tx,
                    stock_uuid,
                    move_uuid,
                    quantity,
                    total_value,
                    price,
                    now,
                )
                .await?;

                insert_recipe_item(&mut tx, recipe_set_uuid, stock_uuid, quantity, now).await?;
            }
        }
    }

    tx.commit().await?;
    Ok(())
}

async fn load_ingredient_map(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<HashMap<String, IngredientInfo>> {
    let rows = sqlx::query!(
        r#"
        SELECT uuid, name, price
        FROM ingredient_catalog
        WHERE deleted_at = 0
        "#
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut map = HashMap::new();
    for row in rows {
        let default_price = row.price.unwrap_or_else(|| Decimal::new(15000, 0));
        map.insert(
            row.name,
            IngredientInfo {
                uuid: row.uuid,
                default_price,
            },
        );
    }
    Ok(map)
}

async fn ensure_category(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    name: &str,
    timestamp: i64,
) -> Result<Uuid> {
    if let Some(row) = sqlx::query!(
        r#"SELECT uuid FROM categories WHERE lower(name) = lower($1) AND deleted_at = 0"#,
        name
    )
    .fetch_optional(&mut **tx)
    .await?
    {
        return Ok(row.uuid);
    }

    let uuid = deterministic_uuid(&format!("category-{}", name));
    sqlx::query!(
        r#"INSERT INTO categories (uuid, name, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $3, 0)"#,
        uuid,
        name,
        timestamp
    )
    .execute(&mut **tx)
    .await?;

    Ok(uuid)
}

async fn upsert_product(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    seed: &ProductSeed,
    category_uuid: Uuid,
    timestamp: i64,
) -> Result<Uuid> {
    if let Some(existing) = sqlx::query!(
        r#"SELECT uuid FROM products WHERE lower(name) = lower($1) AND deleted_at = 0"#,
        seed.name
    )
    .fetch_optional(&mut **tx)
    .await?
    {
        sqlx::query!(
            r#"UPDATE products
               SET category_uuid = $1,
                   price = $2,
                   image_url = $3,
                   updated_at = $4,
                   status = 'ACTIVE'
               WHERE uuid = $5"#,
            category_uuid,
            Decimal::from_f64(seed.price_idr).context("gagal konversi harga")?,
            format!("/uploads/products/{}", seed.image),
            timestamp,
            existing.uuid
        )
        .execute(&mut **tx)
        .await?;
        return Ok(existing.uuid);
    }

    let uuid = deterministic_uuid(&format!("product-{}", seed.name));
    sqlx::query!(
        r#"INSERT INTO products (
                uuid,
                category_uuid,
                name,
                sku,
                price,
                recipe_sets_uuid,
                status,
                image_url,
                created_at,
                updated_at,
                deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, NULL, 'ACTIVE', $6, $7, $7, 0)"#,
        uuid,
        category_uuid,
        seed.name,
        product_sku(seed.name),
        Decimal::from_f64(seed.price_idr).context("gagal konversi harga")?,
        format!("/uploads/products/{}", seed.image),
        timestamp
    )
    .execute(&mut **tx)
    .await?;
    Ok(uuid)
}

async fn upsert_recipe_set(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    seed: &ProductSeed,
    timestamp: i64,
) -> Result<Uuid> {
    // Tentukan effective_from (sekarang atau kemarin) dan effective_to (3/6/12 bulan) secara deterministik per nama
    let one_day_ms: i64 = 86_400_000;
    let name_len = seed.name.len() as i64;
    let effective_from_ts = if name_len % 2 == 0 {
        timestamp
    } else {
        timestamp - one_day_ms
    };
    let effective_to_ts = match name_len % 3 {
        0 => effective_from_ts + 90 * one_day_ms,  // ~3 bulan
        1 => effective_from_ts + 180 * one_day_ms, // ~6 bulan
        _ => effective_from_ts + 365 * one_day_ms, // ~1 tahun
    };

    if let Some(existing) = sqlx::query!(
        r#"SELECT uuid FROM recipe_sets WHERE lower(name) = lower($1) AND deleted_at = 0"#,
        seed.name
    )
    .fetch_optional(&mut **tx)
    .await?
    {
        sqlx::query!(
            r#"UPDATE recipe_sets
               SET yield_quantity = 1,
                   effective_from = $1,
                   effective_to = $2,
                   is_active = true,
                   updated_at = $3
               WHERE uuid = $4"#,
            effective_from_ts,
            effective_to_ts,
            timestamp,
            existing.uuid
        )
        .execute(&mut **tx)
        .await?;
        return Ok(existing.uuid);
    }

    let uuid = deterministic_uuid(&format!("recipe-{}", seed.name));
    sqlx::query!(
        r#"INSERT INTO recipe_sets (
                uuid,
                name,
                yield_quantity,
                effective_from,
                effective_to,
                is_active,
                created_at,
                updated_at,
                deleted_at
            )
            VALUES ($1, $2, 1, $3, $4, true, $5, $5, 0)"#,
        uuid,
        seed.name,
        effective_from_ts,
        effective_to_ts,
        timestamp
    )
    .execute(&mut **tx)
    .await?;
    Ok(uuid)
}

async fn link_product_to_recipe(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    product_uuid: Uuid,
    recipe_set_uuid: Uuid,
    timestamp: i64,
) -> Result<()> {
    // ID: Sinkronkan nama produk dengan nama recipe set yang terhubung.
    // EN: Synchronize product name with the linked recipe set's name.
    sqlx::query(
        r#"UPDATE products
            SET recipe_sets_uuid = $1,
                name = (SELECT name FROM recipe_sets WHERE uuid = $1),
                updated_at = $2
            WHERE uuid = $3"#,
    )
    .bind(recipe_set_uuid)
    .bind(timestamp)
    .bind(product_uuid)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn clear_recipe_items(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    recipe_set_uuid: Uuid,
) -> Result<()> {
    sqlx::query!(
        r#"DELETE FROM recipe_items WHERE recipe_sets_uuid = $1"#,
        recipe_set_uuid
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_production_move(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    move_uuid: Uuid,
    ingredient_uuid: Uuid,
    quantity: Decimal,
    price: Decimal,
    timestamp: i64,
    product_uuid: Uuid,
    label: &str,
) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO ingredient_stock_moves (
                uuid,
                ingredient_catalog_uuid,
                quantity,
                price,
                price_updated_at,
                effective_at,
                expiry_at,
                ref_type,
                ref_uuid,
                name,
                created_at,
                updated_at,
                deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $5, NULL, 'PRODUCTION', $6, $7, $5, $5, 0)
            ON CONFLICT (uuid) DO UPDATE SET name = EXCLUDED.name, updated_at = EXCLUDED.updated_at"#,
        move_uuid,
        ingredient_uuid,
        quantity,
        price,
        timestamp,
        product_uuid,
        label.to_string()
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_production_stock(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    stock_uuid: Uuid,
    move_uuid: Uuid,
    quantity: Decimal,
    total_value: Decimal,
    price: Decimal,
    timestamp: i64,
) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO ingredient_stocks (
                uuid,
                ingredient_stock_moves_uuid,
                total_quantity,
                total_value,
                current_cost,
                avg_cost,
                created_at,
                updated_at,
                deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $5, $6, $6, 0)
            ON CONFLICT (uuid) DO NOTHING"#,
        stock_uuid,
        move_uuid,
        quantity,
        total_value,
        price,
        timestamp
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_recipe_item(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    recipe_set_uuid: Uuid,
    stock_uuid: Uuid,
    quantity: Decimal,
    timestamp: i64,
) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO recipe_items (
                uuid,
                recipe_sets_uuid,
                ingredient_stocks_uuid,
                quantity,
                waste_percent,
                created_at,
                updated_at,
                deleted_at
            )
            VALUES ($1, $2, $3, $4, 0, $5, $5, 0)"#,
        deterministic_uuid(&format!("recipe-item-{}-{}", recipe_set_uuid, stock_uuid)),
        recipe_set_uuid,
        stock_uuid,
        quantity,
        timestamp
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
