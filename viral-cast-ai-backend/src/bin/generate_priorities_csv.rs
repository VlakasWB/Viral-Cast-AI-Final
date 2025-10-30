use anyhow::Result;
use std::{
    env,
    fs::File,
    io::{BufReader, BufWriter},
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let regions_csv = env::var("BMKG_CSV_PATH").unwrap_or_else(|_| "data/regions.csv".to_string());
    let out_csv =
        env::var("OUT_PRIORITIES_CSV_PATH").unwrap_or_else(|_| "data/priorities.csv".to_string());

    let file = File::open(&regions_csv)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(BufReader::new(file));

    // Collect ADM4 codes (codes with 3 dots)
    let mut codes: Vec<String> = Vec::new();
    for rec in rdr.records() {
        let rec = rec?;
        if rec.len() == 0 {
            continue;
        }
        let code = rec.get(0).unwrap().to_string();
        if code.matches('.').count() == 3 {
            codes.push(code);
        }
    }

    // Sort codes ascending for deterministic priority ordering
    codes.sort();

    // Write priorities.csv with headers: region_code,priority,active
    let out = File::create(&out_csv)?;
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(BufWriter::new(out));

    wtr.write_record(["region_code", "priority", "active"])?;
    for (idx, code) in codes.iter().enumerate() {
        let prio = (idx as i32) + 1;
        wtr.write_record([code, &prio.to_string(), "true"])?;
    }
    wtr.flush()?;

    println!("✅ Generated {} with {} rows", out_csv, codes.len());
    Ok(())
}
