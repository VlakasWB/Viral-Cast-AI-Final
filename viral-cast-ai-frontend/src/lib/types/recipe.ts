// Recipe Set Types
export interface RecipeSet {
    uuid: string;
    product_uuid: string;
    name: string;
    // ID: Alias untuk mendukung skema Postman (yield_quantity)
    // EN: Alias to support Postman schema (yield_quantity)
    yield_qty: number;
    yield_quantity?: number;
    effective_from: number; // Unix timestamp in milliseconds
    effective_to: number; // Unix timestamp in milliseconds
    is_active: boolean;
    created_at: number;
    updated_at: number;
    deleted_at?: number | null;
}

export interface CreateRecipeSetRequest {
    name: string;
    // ID: Dukung dua penamaan agar kompatibel (yield_qty & yield_quantity)
    // EN: Support two names for compatibility (yield_qty & yield_quantity)
    yield_qty?: number;
    yield_quantity?: number;
    effective_from?: number;
    effective_to?: number;
    is_active: boolean;
}

export interface UpdateRecipeSetRequest {
    name?: string;
    // ID: Dukung dua penamaan agar kompatibel (yield_qty & yield_quantity)
    // EN: Support two names for compatibility (yield_qty & yield_quantity)
    yield_qty?: number;
    yield_quantity?: number;
    effective_from?: number;
    effective_to?: number;
    is_active?: boolean;
}

// Recipe Item Types
export interface RecipeItem {
    uuid: string;
    // ID: Alias untuk skema lama & Postman baru
    // EN: Aliases for legacy schema & new Postman schema
    recipe_uuid?: string;
    ingredient_uuid?: string;
    qty?: number;
    waste_pct?: number;
    recipe_sets_uuid?: string;
    ingredient_stocks_uuid?: string;
    quantity?: number;
    waste_percent?: number;
    created_at: number;
    updated_at: number;
    deleted_at?: number | null;
    // Optional populated fields
    ingredient_name?: string;
    ingredient_base_uom?: string;
}

export interface CreateRecipeItemRequest {
    // ID: Dukung penamaan lama dan Postman baru
    // EN: Support legacy and Postman naming
    recipe_uuid?: string;
    ingredient_uuid?: string;
    qty?: number;
    waste_pct?: number;
    recipe_sets_uuid?: string;
    ingredient_stocks_uuid?: string;
    quantity?: number;
    waste_percent?: number;
}

export interface UpdateRecipeItemRequest {
    // ID: Dukung penamaan lama dan Postman baru
    // EN: Support legacy and Postman naming
    qty?: number;
    waste_pct?: number;
    quantity?: number;
    waste_percent?: number;
}

// API Response Types
export interface RecipeSetResponse {
    code: number;
    status: string;
    message: string;
    data: {
        recipe_sets: RecipeSet[];
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
    errors: any;
}

export interface SingleRecipeSetResponse {
	code: number;
	status: string;
	message: string;
	data: {
		recipe_set: RecipeSet;
	};
	errors: any;
}

export interface RecipeItemResponse {
    code: number;
    status: string;
    message: string;
    data: {
        recipe_items: RecipeItem[];
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
    errors: any;
}

export interface SingleRecipeItemResponse {
	code: number;
	status: string;
	message: string;
	data: {
		recipe_item: RecipeItem;
	};
	errors: any;
}

export interface DeleteRecipeSetResponse {
	code: number;
	status: string;
	message: string;
	data: {
		deleted: boolean;
		uuid: string;
	};
	errors: any;
}

export interface DeleteRecipeItemResponse {
	code: number;
	status: string;
	message: string;
	data: {
		deleted: boolean;
		uuid: string;
	};
	errors: any;
}

// Query Parameters Types
export interface RecipeSetQueryParams {
    page?: number;
    limit?: number;
    product_uuid?: string;
    name?: string;
    is_active?: boolean;
    search?: string;
}

export interface RecipeItemQueryParams {
    page?: number;
    limit?: number;
    // ID: Dukung filter lama dan Postman
    // EN: Support legacy and Postman filters
    recipe_uuid?: string;
    ingredient_uuid?: string;
    recipe_sets_uuid?: string;
    ingredient_stocks_uuid?: string;
}
