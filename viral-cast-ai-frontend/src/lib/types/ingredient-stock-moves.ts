// ID: Types untuk Ingredient Stock Moves
// EN: Types for Ingredient Stock Moves

export type IngredientStockMoveType =
	| 'PRODUCTION'
	| 'ADJUSTMENT'
	| 'WASTE'
	| 'RETURN'
	| 'PURCHASE'
	| 'TRANSFER'
	| 'CONSUMPTION';

export interface IngredientStockMove {
	uuid: string;
	ingredient_uuid?: string;
	store_uuid?: string;
	quantity: number;
	unit_of_measure_code?: string;
	unit_of_measure_name?: string;
	unit_of_measure_uuid?: string;
	move_type: IngredientStockMoveType | string;
	reference?: string;
	created_at: string;
	updated_at: string;

	// ID: Relasi dengan tabel lain
	// EN: Relations with other tables
	ingredient?: {
		uuid?: string;
		name: string;
		base_uom?: string;
	};
	store?: {
		uuid?: string;
		name: string;
	};
	unit_of_measure?: {
		uuid?: string;
		name: string;
		symbol?: string;
		code?: string;
	};

	// ID: Field tambahan dari API baru
	// EN: Additional fields from the new API
	ingredient_catalog_uuid?: string;
	price?: number;
	price_updated_at?: number | null;
	effective_at?: number | null;
	expiry_at?: number | null;
	ref_type?: string;
	ref_uuid?: string | null;
	name?: string;
}

export interface CreateIngredientStockMoveRequest {
	ingredient_uuid?: string;
	store_uuid?: string;
	quantity: number;
	unit_of_measure_uuid?: string;
	move_type?: IngredientStockMoveType | string;
	reference?: string;
	// API v1 fields
	ingredient_catalog_uuid?: string;
	price?: number;
	price_updated_at?: number | string;
	effective_at?: number | string;
	expiry_at?: number | string | null;
	ref_type?: string;
	ref_uuid?: string | null;
	name?: string;
}

export type UpdateIngredientStockMoveRequest = Partial<CreateIngredientStockMoveRequest>;

export interface IngredientStockMoveFilters {
	store_uuid?: string;
	ingredient_uuid?: string;
	ingredient?: string;
	name?: string;
	move_type?: IngredientStockMoveType | string;
	sort_by?:
		| 'created_at'
		| 'expiry_at'
		| 'quantity'
		| 'move_type'
		| 'name'
		| 'ingredient_name'
		| 'effective_at'
		| string;
	sort_direction?: 'asc' | 'desc';
	search?: string;
	limit?: number;
	offset?: number;
	page?: number;
	ingredient_catalog_uuid?: string;
	ref_type?: string;
	ref_uuid?: string;
	from_date?: string;
	to_date?: string;
}

export interface IngredientStockMovesResponse {
	data: IngredientStockMove[];
	total: number;
	limit: number;
	offset: number;
	page?: number;
	pageCount?: number;
}
