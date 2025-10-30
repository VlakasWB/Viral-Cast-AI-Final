export interface Category {
	uuid: string;
	name: string;
	created_at: number;
	updated_at: number;
}

export interface CreateCategoryRequest {
	name: string;
}

export interface UpdateCategoryRequest {
	name: string;
}

export interface CategoryResponse {
	code: number;
	status: string;
	message: string;
	data: Category[];
	errors: Record<string, any>;
}

export interface SingleCategoryResponse {
	code: number;
	status: string;
	message: string;
	data: Category;
	errors: Record<string, any>;
}

export interface DeleteCategoryResponse {
	code: number;
	status: string;
	message: string;
	data: Record<string, any>;
	errors: Record<string, any>;
}
