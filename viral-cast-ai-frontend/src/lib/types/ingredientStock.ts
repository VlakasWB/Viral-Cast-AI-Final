import type { Ingredient } from './ingredient';

export interface IngredientStock {
	uuid: string;
	ingredient_uuid: string;
	ingredient: Ingredient;
	total_quantity: number;
	unit_of_measure_code?: string;
	unit_of_measure_name?: string;
	total_value: number;
	current_cost: number;
	avg_cost: number;
	created_at: number;
	updated_at: number;

	ingredient_stock_move_uuid?: string;
	ingredient_catalog_uuid?: string;
	ingredient_name?: string;
}

export interface IngredientStockResponse {
	data: IngredientStock[];
	meta: {
		total: number;
		current_page: number;
		last_page: number;
		per_page: number;
	};
	page?: number;
	pageCount?: number;
	limit?: number;
}
