podman-dev-up:
	podman compose up -d

podman-dev-down:
	podman compose down

podman-dev-down-volumes:
	podman compose down --volumes

migrate-up:
	sqlx migrate run

migrate-down:
	sqlx migrate revert

start-server:
	cargo watch -q -c -w src/ -x run

install:
	cargo add axum
	cargo add tokio -F full
	cargo add tower-http -F "cors"
	cargo add serde_json
	cargo add serde -F derive
	cargo add chrono -F serde
	cargo add dotenv
	cargo add uuid -F "serde v4"
	cargo add sqlx -F "runtime-async-std-native-tls postgres chrono uuid"
	cargo add thiserror
	cargo add pretty_env_logger
	cargo add async-trait
	cargo add axum-extra -F cookie
	cargo add time
	cargo add jsonwebtoken
	cargo add argon2
	cargo add rand_core -F std
	cargo add base64
	cargo add redis -F tokio-comp
	# HotReload
	cargo install cargo-watch
	# SQLX-CLI
	cargo install sqlx-cli

# --- Seeding helpers ---
.PHONY: seed-master-roles seed-regions seed-bmkg-db seed-priorities seed-bmkg-csv seed-all seed-uoms seed-ingredient-catalog seed-ingredient-stock-moves seed-ingredient-stocks seed-products-recipes

seed-master-roles:
	cargo run --bin seed_master_roles

seed-regions:
	cmd /c "set REGION_CSV_PATH=$(CURDIR)/data/regions.csv && cargo run --bin seed_regions"

seed-bmkg-db:
	cargo run --bin seed_bmkg_area

seed-priorities:
	cmd /c "set BMKG_CSV_PATH=$(CURDIR)/data/regions.csv && set OUT_PRIORITIES_CSV_PATH=$(CURDIR)/data/priorities.csv && cargo run --bin generate_priorities_csv"

seed-bmkg-csv:
	cmd /c "set BMKG_CSV_PATH=$(CURDIR)/data/regions.csv && set BMKG_PRIORITIES_CSV_PATH=$(CURDIR)/data/priorities.csv && cargo run --bin seed_bmkg_area_from_csv"

seed-uoms:
	cargo run --bin seed_units_of_measure

seed-ingredient-catalog:
	cargo run --bin seed_ingredient_catalog

seed-ingredient-stock-moves:
	cargo run --bin seed_ingredient_stock_moves

seed-ingredient-stocks:
	cargo run --bin seed_ingredient_stocks

seed-products-recipes:
	cargo run --bin seed_products_recipes

# One-shot: run all seeds in order (roles -> regions -> bmkg areas -> priorities -> upsert priorities -> uoms -> ingredient catalog -> stock moves -> stocks -> products/recipes)
seed-all:
	$(MAKE) seed-master-roles
	$(MAKE) seed-regions
	$(MAKE) seed-bmkg-db
	$(MAKE) seed-priorities
	$(MAKE) seed-bmkg-csv
	$(MAKE) seed-uoms
	$(MAKE) seed-ingredient-catalog
	$(MAKE) seed-ingredient-stock-moves
	$(MAKE) seed-ingredient-stocks
	$(MAKE) seed-products-recipes
