import type {
	Category,
	CreateCategoryRequest,
	UpdateCategoryRequest,
	CategoryResponse,
	SingleCategoryResponse,
	DeleteCategoryResponse
} from '$lib/types/category.js';
import { browser } from '$app/environment';
import { API_BASE_URL } from '$env/static/private';

class CategoryApiError extends Error {
	constructor(
		message: string,
		public status: number,
		public response?: any
	) {
		super(message);
		this.name = 'CategoryApiError';
	}
}

async function makeRequest<T>(
	endpoint: string,
	options: RequestInit = {},
	cookies?: string,
	fetchImpl: typeof fetch = fetch
): Promise<T> {
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
			throw new CategoryApiError(
				errorData.message || `HTTP ${response.status}: ${response.statusText}`,
				response.status,
				errorData
			);
		}

		return await response.json();
	} catch (error) {
		if (error instanceof CategoryApiError) {
			throw error;
		}
		throw new CategoryApiError(
			error instanceof Error ? error.message : 'Network error occurred',
			0
		);
	}
}

// GET /api/v1/categories
export async function getCategoriesApi(
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<CategoryResponse> {
	return makeRequest<CategoryResponse>(
		'/api/v1/categories',
		{
			method: 'GET'
		},
		cookies,
		fetchImpl
	);
}

// GET /api/v1/categories/{uuid}
export async function getCategoryApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleCategoryResponse> {
	return makeRequest<SingleCategoryResponse>(
		`/api/v1/categories/${uuid}`,
		{
			method: 'GET'
		},
		cookies,
		fetchImpl
	);
}

// POST /api/v1/categories
export async function createCategoryApi(
	data: CreateCategoryRequest,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleCategoryResponse> {
	return makeRequest<SingleCategoryResponse>(
		'/api/v1/categories',
		{
			method: 'POST',
			body: JSON.stringify(data)
		},
		cookies,
		fetchImpl
	);
}

// PATCH /api/v1/categories/{uuid}
export async function updateCategoryApi(
	uuid: string,
	data: UpdateCategoryRequest,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleCategoryResponse> {
	return makeRequest<SingleCategoryResponse>(
		`/api/v1/categories/${uuid}`,
		{
			method: 'PATCH',
			body: JSON.stringify(data)
		},
		cookies,
		fetchImpl
	);
}

// DELETE /api/v1/categories/{uuid}
export async function deleteCategoryApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<DeleteCategoryResponse> {
	return makeRequest<DeleteCategoryResponse>(
		`/api/v1/categories/${uuid}`,
		{
			method: 'DELETE'
		},
		cookies,
		fetchImpl
	);
}

export { CategoryApiError };
