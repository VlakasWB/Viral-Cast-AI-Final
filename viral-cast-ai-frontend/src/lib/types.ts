// [ID] Tipe data utama untuk produk.
// [EN] Main data type for product.
export type Product = {
	id: string;
	name: string;
	price: number;
	stock: number;
	active: boolean;
};

// API Response type
export interface ApiResponse<T = any> {
	code: number;
	status: string;
	message: string;
	data: T;
	errors?: any;
}

// Re-export Product from product types for consistency
export type { Product as ProductType } from './types/product';
