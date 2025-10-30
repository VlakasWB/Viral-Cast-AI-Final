use anyhow::Result;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    seed_uoms(&pool).await?;
    println!("✅ UOM seeding done");
    Ok(())
}

#[derive(Debug, Clone)]
struct UomSeed {
    uuid: Uuid,
    name: &'static str,
    code: &'static str,
}

const BASE_TIMESTAMP_MS: u64 = 1_720_000_000_000;

fn uuid_v7_from_code(base_ts: u64, ordinal: usize, code: &str) -> Uuid {
    let ts = base_ts + ordinal as u64;
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hasher.update((ordinal as u64).to_be_bytes());
    let digest = hasher.finalize();

    let mut rand_bytes = [0u8; 16];
    rand_bytes.copy_from_slice(&digest[..16]);
    let rand = u128::from_be_bytes(rand_bytes);
    let rand_a = ((rand >> 116) & 0x0fff) as u16;
    let rand_b = rand & ((1u128 << 62) - 1);

    let mut value = ((ts as u128) & ((1u128 << 48) - 1)) << 80;
    value |= (0b0111u128) << 76;
    value |= (rand_a as u128) << 64;
    value |= (0b10u128) << 62;
    value |= rand_b;

    Uuid::from_u128(value)
}

const RAW_UOM_DATA: &[(&str, &str, &str)] = &[
    // Mass Units
    ("Kilogram", "kg", "Unit of mass in the metric system"),
    ("Gram", "g", "Smaller unit of mass, 1/1000 of a kilogram"),
    (
        "Milligram",
        "mg",
        "Very small unit of mass, 1/1000 of a gram",
    ),
    ("Ton", "ton", "Metric ton, equal to 1000 kilograms"),
    (
        "Pound",
        "lb",
        "Imperial unit of mass, approximately 0.454 kg",
    ),
    ("Ounce", "oz", "Imperial unit of mass, 1/16 of a pound"),
    ("Quintal", "q", "100 kilograms"),
    // Volume Units
    ("Liter", "L", "Unit of volume in the metric system"),
    (
        "Milliliter",
        "ml",
        "Smaller unit of volume, 1/1000 of a liter",
    ),
    (
        "Cubic Meter",
        "m³",
        "Volume of a cube with 1m sides, equals 1000 liters",
    ),
    (
        "Cubic Centimeter",
        "cm³",
        "Volume of a cube with 1cm sides, equals 1 milliliter",
    ),
    (
        "Gallon",
        "gal",
        "Imperial unit of volume, approximately 3.785 liters",
    ),
    (
        "Fluid Ounce",
        "fl oz",
        "Imperial unit of volume, 1/128 of a gallon",
    ),
    ("Cup", "cup", "Culinary unit of volume, 8 fluid ounces"),
    (
        "Tablespoon",
        "tbsp",
        "Culinary unit of volume, 1/16 of a cup",
    ),
    (
        "Teaspoon",
        "tsp",
        "Culinary unit of volume, 1/3 of a tablespoon",
    ),
    ("Pint", "pt", "Imperial unit of volume, 16 fluid ounces"),
    ("Quart", "qt", "Imperial unit of volume, 32 fluid ounces"),
    // Length Units
    ("Meter", "m", "Basic unit of length in the metric system"),
    ("Centimeter", "cm", "1/100 of a meter"),
    ("Millimeter", "mm", "1/1000 of a meter"),
    ("Inch", "in", "Imperial unit of length, 2.54 centimeters"),
    ("Foot", "ft", "Imperial unit of length, 12 inches"),
    // Area Units
    ("Square Meter", "m²", "Area of a square with 1m sides"),
    ("Hectare", "ha", "10,000 square meters"),
    (
        "Acre",
        "acre",
        "Imperial unit of area, approximately 4047 square meters",
    ),
    // Count Units
    ("Pieces", "pcs", "Individual countable items"),
    ("Dozen", "dz", "A group of twelve items"),
    ("Pair", "pr", "A set of two items"),
    (
        "Bunch",
        "bn",
        "Tied bundle of herbs or greens (e.g., scallions, celery)",
    ),
    (
        "Bundle",
        "bdl",
        "A collection of items tied or wrapped together",
    ),
    ("Box", "box", "A container for storing items"),
    ("Carton", "ctn", "A container made of cardboard"),
    ("Pack", "pk", "A small package or bundle"),
    (
        "Jar",
        "jar",
        "Glass container commonly used for sauces or pickles",
    ),
    (
        "Slice",
        "slc",
        "Thin, flat piece of an item (e.g., cake, bread, cheese)",
    ),
    (
        "Stick",
        "stk",
        "Long, thin piece (e.g., butter stick, cinnamon stick)",
    ),
    (
        "Roll",
        "roll",
        "Coiled or wrapped item (e.g., sushi roll, pastry roll)",
    ),
    // Utility Units
    ("Bottle", "btl", "Glass or plastic container for liquids"),
    ("Can", "can", "Metal container, commonly for preserved food"),
    (
        "Packet",
        "pkt",
        "Small sealed package, often for seasonings",
    ),
    (
        "Tray",
        "try",
        "Flat container used for serving or displaying items",
    ),
    (
        "Bucket",
        "bkt",
        "Container with a handle, used for liquids or bulk items",
    ),
    (
        "Bar",
        "bar",
        "Solid rectangular block (e.g., chocolate bar, soap bar)",
    ),
    (
        "Stick Pack",
        "spk",
        "Single-use stick-shaped package (e.g., sugar, coffee mix)",
    ),
    ("Sachet", "sct", "Small sealed packet for single servings"),
    ("Cup Metric", "cupm", "Metric cup, standardized at 250 ml"),
    ("Scoops", "scp", "Rounded measuring spoon for powders"),
    ("Serving", "srv", "Standardized portion size"),
    ("Portion", "prt", "Another term for serving size"),
    ("Sprig", "spg", "Small stem of herbs"),
    (
        "Leaf",
        "leaf",
        "Individual leaf (e.g., basil leaf, lettuce leaf)",
    ),
    ("Clove", "clv", "Single segment of garlic or spice"),
    ("Head", "hd", "Whole head of produce"),
    (
        "Whole",
        "whl",
        "Entire item, often used for poultry or fruits",
    ),
    (
        "Sendok Makan",
        "sdm",
        "Indonesian tablespoon, approximately 15 ml",
    ),
    (
        "Sendok Teh",
        "sdt",
        "Indonesian teaspoon, approximately 5 ml",
    ),
    (
        "Bungkus",
        "bks",
        "Package or wrapper, commonly used for spices or ingredients",
    ),
    ("Ikat", "ikt", "Bundle of vegetables or herbs tied together"),
];

async fn seed_uoms(pool: &sqlx::PgPool) -> Result<()> {
    let now_ms: i64 = chrono::Utc::now().timestamp_millis();

    let seeds: Vec<UomSeed> = RAW_UOM_DATA
        .iter()
        .enumerate()
        .map(|(idx, (name, code, _description))| UomSeed {
            uuid: uuid_v7_from_code(BASE_TIMESTAMP_MS, idx, code),
            name,
            code,
        })
        .collect();

    let mut tx = pool.begin().await?;

    for s in seeds.into_iter() {
        sqlx::query!(
            r#"
            INSERT INTO units_of_measure (uuid, code, name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $4)
            ON CONFLICT (code) DO UPDATE SET
              name = EXCLUDED.name,
              updated_at = EXCLUDED.updated_at
            "#,
            s.uuid,
            s.code,
            s.name,
            now_ms,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
