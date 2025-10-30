#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL is required}"

echo "[entrypoint] running migrations…"
sqlx migrate run

if [ "${RUN_SEED:-true}" = "true" ]; then
  CSV="${REGION_CSV_PATH:-/app/data/regions.csv}"
  echo "[entrypoint] seeding regions from ${CSV} …"
  seed_regions
else
  echo "[entrypoint] skipping seeding (RUN_SEED=false)"
fi

# UOMs seeding (idempotent)
if [ "${RUN_UOMS_SEED:-true}" = "true" ]; then
  echo "[entrypoint] seeding UOMs…"
  seed_uoms || {
    echo "[entrypoint] seed_uoms failed";
    exit 1;
  }
else
  echo "[entrypoint] skipping UOMs seeding (RUN_UOMS_SEED=false)"
fi

# BMKG seeding: DB areas + priorities from CSV (idempotent)
if [ "${RUN_BMKG_SEED:-true}" = "true" ]; then
  echo "[entrypoint] seeding bmkg_area from master DB regions…"
  seed_bmkg_area || {
    echo "[entrypoint] seed_bmkg_area failed";
    exit 1;
  }

  BMKG_CSV="${BMKG_CSV_PATH:-/app/data/regions.csv}"
  BMKG_PRIOS_CSV="${BMKG_PRIORITIES_CSV_PATH:-/app/data/priorities.csv}"
  echo "[entrypoint] BMKG CSV path: ${BMKG_CSV}"
  echo "[entrypoint] BMKG priorities CSV path: ${BMKG_PRIOS_CSV}"

  # Generate priorities CSV when requested or missing (default true)
  if [ "${RUN_GENERATE_PRIORITIES:-true}" = "true" ] || [ ! -f "$BMKG_PRIOS_CSV" ]; then
    echo "[entrypoint] generating priorities CSV…"
    generate_priorities_csv || {
      echo "[entrypoint] generate_priorities_csv failed";
      exit 1;
    }
  else
    echo "[entrypoint] using existing priorities CSV (no generation)"
  fi

  echo "[entrypoint] upserting bmkg_area_priority from priorities CSV…"
  seed_bmkg_area_from_csv || {
    echo "[entrypoint] seed_bmkg_area_from_csv failed";
    exit 1;
  }
else

fi

echo "[entrypoint] starting server…"
exec viral_cast_ai_backend
