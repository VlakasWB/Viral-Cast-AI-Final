/**
 * Struktur data satuan dasar (Unit of Measure) yang terasosiasi dengan ingredient catalog.
 * ID: Struktur data satuan dasar yang terkait dengan katalog bahan.
 * EN: Unit of Measure structure associated with an ingredient catalog.
 */
export interface IngredientCatalogUom {
	uuid: string;
	code: string;
	name: string;
}

/**
 * Tipe data untuk Ingredient Catalog.
 * ID: Tipe data untuk Katalog Bahan.
 * EN: Type definition for Ingredient Catalog.
 */
export interface IngredientCatalog {
	uuid: string;
	name: string;
	description?: string;
	base_uom: IngredientCatalogUom;
	minimum_stock: number;
	shelf_life_days: number;
	created_at: number;
	updated_at: number;
	deleted_at?: number | null;
}

/**
 * Tipe data generik untuk respons API dengan pagination.
 * ID: Tipe data generik respons API dengan paginasi.
 * EN: Generic type definition for paginated API responses.
 */
export interface PaginatedResponse<T> {
	items: T[];
	total: number;
	page: number;
	pageCount: number;
}
