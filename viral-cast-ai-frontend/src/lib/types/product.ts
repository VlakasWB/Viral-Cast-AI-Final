export interface Product {
    uuid: string;
    name: string;
    category_uuid: string; // UUID reference to Category
    sku: string;
    price: string; // Decimal as string
    current_recipe_uuid: string | null;
    // ID: Tambahkan dukungan untuk skema Postman (recipe_sets_uuid)
    // EN: Add support for Postman schema (recipe_sets_uuid)
    recipe_sets_uuid?: string | null;
    status: 'ACTIVE' | 'INACTIVE';
    image_url: string | null;
    created_at: number;
    updated_at: number;
}

export interface CreateProductRequest {
    name: string;
    category_uuid: string; // UUID reference to Category
    sku: string;
    price: number;
    current_recipe_uuid?: string | null;
    // ID: Dukungan Postman, alternatif dari current_recipe_uuid
    // EN: Postman support, alternative to current_recipe_uuid
    recipe_sets_uuid?: string | null;
    status: 'ACTIVE' | 'INACTIVE';
    image_url?: string;
}

export interface UpdateProductRequest {
    name?: string;
    category_uuid?: string; // UUID reference to Category
    sku?: string;
    price?: number;
    current_recipe_uuid?: string | null;
    // ID: Dukungan Postman, alternatif dari current_recipe_uuid
    // EN: Postman support, alternative to current_recipe_uuid
    recipe_sets_uuid?: string | null;
    status?: 'ACTIVE' | 'INACTIVE';
    image_url?: string;
}

export interface ProductResponse {
    code: number;
    status: string;
    message: string;
    data: {
        products: Product[];
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

export interface SingleProductResponse {
	code: number;
	status: string;
	message: string;
	data: {
		product: Product;
	};
	errors: any;
}

export interface ImageUploadResponse {
	code: number;
	status: string;
	message: string;
	data: {
		file_size: number;
		filename: string;
		image_url: string;
		original_filename: string;
	};
	errors: any;
}

export interface ImageDeleteRequest {
	filename: string;
}

export interface ImageDeleteResponse {
	code: number;
	status: string;
	message: string;
}
