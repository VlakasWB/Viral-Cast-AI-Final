import type {
	Product,
	CreateProductRequest,
	UpdateProductRequest,
	ProductResponse,
	SingleProductResponse,
	ImageUploadResponse,
	ImageDeleteRequest,
	ImageDeleteResponse
} from '$lib/types/product.js';
import { browser } from '$app/environment';

// Get API base URL - works on both server and client
const getApiBaseUrl = () => {
	try {
		return process.env.API_BASE_URL || 'http://localhost:12000';
	} catch {
		return 'http://localhost:12000';
	}
};

class ProductApiError extends Error {
	constructor(
		message: string,
		public status: number,
		public response?: any
	) {
		super(message);
		this.name = 'ProductApiError';
	}
}

async function makeRequest<T>(
	endpoint: string,
	options: RequestInit = {},
	cookies?: string,
	fetchImpl: typeof fetch = fetch
): Promise<T> {
	const API_BASE_URL = getApiBaseUrl();
	const url = `${API_BASE_URL}${endpoint}`;

	// Get access token from cookie (browser or server)
	let accessToken: string | undefined;

	if (browser) {
		// Browser environment
		accessToken = document.cookie
			.split('; ')
			.find((row) => row.startsWith('access_token='))
			?.split('=')[1];
	} else if (cookies) {
		// Server environment - cookies passed from server
		accessToken = cookies
			.split('; ')
			.find((row) => row.startsWith('access_token='))
			?.split('=')[1];
	}

	const defaultHeaders: HeadersInit = {
		'Content-Type': 'application/json',
		...(accessToken && { Authorization: `Bearer ${accessToken}` })
	};

	const config: RequestInit = {
		...options,
		headers: {
			...defaultHeaders,
			...options.headers
		},
		credentials: 'include'
	};

	try {
		const response = await fetchImpl(url, config);

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({}));
			throw new ProductApiError(
				errorData.message || `HTTP ${response.status}: ${response.statusText}`,
				response.status,
				errorData
			);
		}

		// Handle 204 No Content for delete
		if (response.status === 204) {
			return {} as T;
		}

		return await response.json();
	} catch (error) {
		if (error instanceof ProductApiError) {
			throw error;
		}
		throw new ProductApiError(error instanceof Error ? error.message : 'Network error occurred', 0);
	}
}

// GET /api/v1/products
export async function getProductsApi(
    params?: {
        page?: number;
        limit?: number;
        sort_by?: string;
        sort_order?: 'asc' | 'desc';
        is_active?: boolean;
        search?: string;
        // ID: Tambahkan filter yang digunakan di Postman
        // EN: Add filters used in Postman
        category_uuid?: string;
        status?: 'ACTIVE' | 'INACTIVE';
    },
    cookies?: string,
    fetchImpl?: typeof fetch
): Promise<ProductResponse> {
    const searchParams = new URLSearchParams();

    if (params?.page) searchParams.set('page', params.page.toString());
    if (params?.limit) searchParams.set('limit', params.limit.toString());
    if (params?.sort_by) searchParams.set('sort_by', params.sort_by);
    if (params?.sort_order) searchParams.set('sort_order', params.sort_order);
    if (params?.is_active !== undefined) searchParams.set('is_active', params.is_active.toString());
    if (params?.search) searchParams.set('search', params.search);
    if (params?.category_uuid) searchParams.set('category_uuid', params.category_uuid);
    if (params?.status) searchParams.set('status', params.status);

	const queryString = searchParams.toString();
	const endpoint = `/api/v1/products${queryString ? `?${queryString}` : ''}`;

	return makeRequest<ProductResponse>(
		endpoint,
		{
			method: 'GET'
		},
		cookies,
		fetchImpl
	);
}

// GET /api/v1/products/{uuid}
export async function getProductApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleProductResponse> {
	return makeRequest<SingleProductResponse>(
		`/api/v1/products/${uuid}`,
		{
			method: 'GET'
		},
		cookies,
		fetchImpl
	);
}

// POST /api/v1/products
export async function createProductApi(
	data: CreateProductRequest,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleProductResponse> {
	return makeRequest<SingleProductResponse>(
		'/api/v1/products',
		{
			method: 'POST',
			body: JSON.stringify(data)
		},
		cookies,
		fetchImpl
	);
}

// PUT /api/v1/products/{uuid}
export async function updateProductApi(
	uuid: string,
	data: UpdateProductRequest,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleProductResponse> {
	return makeRequest<SingleProductResponse>(
		`/api/v1/products/${uuid}`,
		{
			method: 'PUT',
			body: JSON.stringify(data)
		},
		cookies,
		fetchImpl
	);
}

// DELETE /api/v1/products/{uuid}
export async function deleteProductApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<void> {
	return makeRequest<void>(
		`/api/v1/products/${uuid}`,
		{
			method: 'DELETE'
		},
		cookies,
		fetchImpl
	);
}

// POST /api/v1/images/upload/product
export async function uploadProductImageApi(
    file: File,
    cookies?: string,
    fetchImpl?: typeof fetch
): Promise<ImageUploadResponse> {
    const API_BASE_URL = getApiBaseUrl();
    const url = `${API_BASE_URL}/api/v1/images/upload/product`;

	// Get access token from cookie (browser or server)
	let accessToken: string | undefined;

	if (browser) {
		accessToken = document.cookie
			.split('; ')
			.find((row) => row.startsWith('access_token='))
			?.split('=')[1];
	} else if (cookies) {
		accessToken = cookies
			.split('; ')
			.find((row) => row.startsWith('access_token='))
			?.split('=')[1];
	}

	const formData = new FormData();
	formData.append('image', file);

	const config: RequestInit = {
		method: 'POST',
		body: formData,
		headers: {
			...(accessToken && { Authorization: `Bearer ${accessToken}` })
		},
		credentials: 'include'
	};

    try {
        const response = await (fetchImpl || fetch)(url, config);

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({}));
			throw new ProductApiError(
				errorData.message || `HTTP ${response.status}: ${response.statusText}`,
				response.status,
				errorData
			);
		}

		return await response.json();
	} catch (error) {
		if (error instanceof ProductApiError) {
			throw error;
		}
		throw new ProductApiError(error instanceof Error ? error.message : 'Network error occurred', 0);
	}
}

// DELETE /api/v1/images/delete/product
export async function deleteProductImageApi(
	data: ImageDeleteRequest,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<ImageDeleteResponse> {
	return makeRequest<ImageDeleteResponse>(
		'/api/v1/images/delete/product',
		{
			method: 'DELETE',
			body: JSON.stringify(data)
		},
		cookies,
		fetchImpl
	);
}

export { ProductApiError };
