export interface Ingredient {
	uuid: string;
	name: string;
	base_uom: {
		uuid: string;
		code: string;
		name: string;
	};
	min_stock: string; // Decimal as string
	shelf_life_days: number;
	created_at: number;
	updated_at: number;
}

export interface CreateIngredientRequest {
	name: string;
	base_uom: string; // UUID reference to UOM
	min_stock: number;
	shelf_life_days: number;
}

export interface UpdateIngredientRequest {
	name?: string;
	base_uom?: string; // UUID reference to UOM
	min_stock?: number;
	shelf_life_days?: number;
}

export interface IngredientResponse {
    code: number;
    status: string;
    message: string;
    data: {
        ingredients: Ingredient[];
        pagination?: {
            page: number;
            limit: number;
            total: number;
            total_pages: number;
            has_prev?: boolean;
            has_next?: boolean;
            prev_page?: number | null;
            next_page?: number | null;
            total_displayed_records?: number;
            total_remaining_records?: number;
        };
    };
    errors: Record<string, any>;
}

export interface SingleIngredientResponse {
	code: number;
	status: string;
	message: string;
	data: Ingredient;
	errors: Record<string, any>;
}

export interface DeleteIngredientResponse {
	message: string;
	status: string;
}
