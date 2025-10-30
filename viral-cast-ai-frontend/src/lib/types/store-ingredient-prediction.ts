export interface StoreIngredientPredictionWeather {
	summary?: string;
	temp_min_c?: number;
	temp_max_c?: number;
	humidity_avg?: number;
	precipitation_total_mm?: number;
}

export interface StoreIngredientPredictionItem {
	ingredient_catalog_uuid?: string;
	ingredient_name?: string;
	unit_of_measure_code?: string;
	unit_of_measure_name?: string;
	restock_label?: string;
	restock_probability?: number;
	recommended_restock_qty?: number;
	current_stock_qty?: number;
	minimum_stock_qty?: number;
	llm_reasoning?: string;
	forecast_error_margin_pct?: number;
}

export interface StoreIngredientPredictionsData {
	store_uuid?: string;
	region_code?: string;
	timezone?: string;
	weather?: StoreIngredientPredictionWeather;
	ingredients?: StoreIngredientPredictionItem[];
	llm_model?: string;
	llm_summary?: string;
	generated_at_ms?: number;
}
