import type {
	Ingredient,
	CreateIngredientRequest,
	UpdateIngredientRequest,
	IngredientResponse,
	SingleIngredientResponse,
	DeleteIngredientResponse
} from '$lib/types/ingredient.js';
import { browser } from '$app/environment';

// Get API base URL - works on both server and client
const getApiBaseUrl = () => {
	try {
		return process.env.API_BASE_URL || 'http://localhost:12000';
	} catch {
		return 'http://localhost:12000';
	}
};

class IngredientApiError extends Error {
	constructor(
		message: string,
		public status: number,
		public response?: any
	) {
		super(message);
		this.name = 'IngredientApiError';
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
            const isUnauthorized = response.status === 401;
            const msg = String(errorData?.message || '');
            const tokenInvalid = /Token is invalid|session has expired/i.test(msg);
            if (browser && (isUnauthorized || tokenInvalid)) {
                const from = `${window.location.pathname}${window.location.search}`;
                window.location.href = `/login?from=${encodeURIComponent(from)}`;
            }
            throw new IngredientApiError(
                errorData.message || `HTTP ${response.status}: ${response.statusText}`,
                response.status,
                errorData
            );
        }

		return await response.json();
	} catch (error) {
		if (error instanceof IngredientApiError) {
			throw error;
		}
		throw new IngredientApiError(
			error instanceof Error ? error.message : 'Network error occurred',
			0
		);
	}
}

// GET /api/v1/ingredients
export async function getIngredientsApi(
    params?: { page?: number; limit?: number; search?: string },
    cookies?: string,
    fetchImpl?: typeof fetch
): Promise<IngredientResponse> {
    const searchParams = new URLSearchParams();
    if (params?.page) searchParams.set('page', params.page.toString());
    if (params?.limit) searchParams.set('limit', params.limit.toString());
    if (params?.search) searchParams.set('search', params.search);

    const query = searchParams.toString();
    const endpoint = query ? `/api/v1/ingredients?${query}` : '/api/v1/ingredients';

    return makeRequest<IngredientResponse>(
        endpoint,
        {
            method: 'GET'
        },
        cookies,
        fetchImpl
    );
}

// GET /api/v1/ingredients/{uuid}
export async function getIngredientApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleIngredientResponse> {
	return makeRequest<SingleIngredientResponse>(
		`/api/v1/ingredients/${uuid}`,
		{
			method: 'GET'
		},
		cookies,
		fetchImpl
	);
}

// POST /api/v1/ingredients
export async function createIngredientApi(
	data: CreateIngredientRequest,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleIngredientResponse> {
	return makeRequest<SingleIngredientResponse>(
		'/api/v1/ingredients',
		{
			method: 'POST',
			body: JSON.stringify(data)
		},
		cookies,
		fetchImpl
	);
}

// PUT /api/ingredients/{uuid} (Note: API uses PUT instead of PATCH)
export async function updateIngredientApi(
	uuid: string,
	data: UpdateIngredientRequest,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleIngredientResponse> {
	return makeRequest<SingleIngredientResponse>(
		`/api/ingredients/${uuid}`,
		{
			method: 'PUT',
			body: JSON.stringify(data)
		},
		cookies,
		fetchImpl
	);
}

// DELETE /api/ingredients/{uuid}
export async function deleteIngredientApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<DeleteIngredientResponse> {
	return makeRequest<DeleteIngredientResponse>(
		`/api/ingredients/${uuid}`,
		{
			method: 'DELETE'
		},
		cookies,
		fetchImpl
	);
}

export { IngredientApiError };
