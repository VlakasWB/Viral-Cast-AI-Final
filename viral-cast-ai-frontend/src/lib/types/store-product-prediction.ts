export interface StoreProductPredictionWeather {
	summary?: string;
	temp_min_c?: number;
	temp_max_c?: number;
	humidity_avg?: number;
	precipitation_total_mm?: number;
}

export interface StoreProductPredictionProduct {
	product_uuid?: string;
	product_name?: string;
	product_sku?: string;
	demand_label?: string;
	demand_probability?: number;
	recommended_stock_qty?: number;
	llm_reasoning?: string;
}

export interface StoreProductPredictionsData {
	store_uuid?: string;
	region_code?: string;
	timezone?: string;
	weather?: StoreProductPredictionWeather;
	products?: StoreProductPredictionProduct[];
	llm_model?: string;
	llm_summary?: string;
	generated_at_ms?: number;
}
