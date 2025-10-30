import type {
	RecipeSet,
	RecipeItem,
	CreateRecipeSetRequest,
	UpdateRecipeSetRequest,
	CreateRecipeItemRequest,
	UpdateRecipeItemRequest,
	RecipeSetResponse,
	SingleRecipeSetResponse,
	RecipeItemResponse,
	SingleRecipeItemResponse,
	DeleteRecipeSetResponse,
	DeleteRecipeItemResponse,
	RecipeSetQueryParams,
	RecipeItemQueryParams
} from '$lib/types/recipe.js';
import { browser } from '$app/environment';

// Get API base URL - works on both server and client
const getApiBaseUrl = () => {
	try {
		return process.env.API_BASE_URL || 'http://localhost:12000';
	} catch {
		return 'http://localhost:12000';
	}
};

class RecipeApiError extends Error {
	constructor(
		message: string,
		public status: number,
		public response?: any
	) {
		super(message);
		this.name = 'RecipeApiError';
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
			throw new RecipeApiError(
				errorData.message || `HTTP ${response.status}: ${response.statusText}`,
				response.status,
				errorData
			);
		}

		return await response.json();
	} catch (error) {
		if (error instanceof RecipeApiError) {
			throw error;
		}
		throw new RecipeApiError(error instanceof Error ? error.message : 'Network error occurred', 0);
	}
}

// ========== RECIPE SET API FUNCTIONS ==========

// GET /api/recipe-sets
export async function getRecipeSetsApi(
    params?: RecipeSetQueryParams,
    cookies?: string,
    fetchImpl?: typeof fetch
): Promise<RecipeSetResponse> {
    const searchParams = new URLSearchParams();

    if (params?.page) searchParams.set('page', params.page.toString());
    if (params?.limit) searchParams.set('limit', params.limit.toString());
    if (params?.product_uuid) searchParams.set('product_uuid', params.product_uuid);
    if (params?.name) searchParams.set('name', params.name);
    if (params?.is_active !== undefined) searchParams.set('is_active', params.is_active.toString());
    if (params?.search) searchParams.set('search', params.search);

	const queryString = searchParams.toString();
	const endpoint = `/api/recipe-sets${queryString ? `?${queryString}` : ''}`;

	return makeRequest<RecipeSetResponse>(
		endpoint,
		{
			method: 'GET'
		},
		cookies,
		fetchImpl
	);
}

// GET /api/recipe-sets/{uuid}
export async function getRecipeSetApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleRecipeSetResponse> {
	return makeRequest<SingleRecipeSetResponse>(
		`/api/recipe-sets/${uuid}`,
		{
			method: 'GET'
		},
		cookies,
		fetchImpl
	);
}

// POST /api/recipe-sets
export async function createRecipeSetApi(
    data: CreateRecipeSetRequest,
    cookies?: string,
    fetchImpl?: typeof fetch
): Promise<SingleRecipeSetResponse> {
    // ID: Mapping field ke skema Postman (yield_quantity)
    // EN: Map fields to Postman schema (yield_quantity)
    const mapped = {
        name: data.name,
        yield_quantity: (data as any).yield_quantity ?? data.yield_qty,
        effective_from: data.effective_from ?? null,
        effective_to: data.effective_to ?? null,
        is_active: data.is_active
    };
    return makeRequest<SingleRecipeSetResponse>(
        '/api/recipe-sets',
        {
            method: 'POST',
            body: JSON.stringify(mapped)
        },
        cookies,
        fetchImpl
    );
}

// PUT /api/recipe-sets/{uuid}
export async function updateRecipeSetApi(
    uuid: string,
    data: UpdateRecipeSetRequest,
    cookies?: string,
    fetchImpl?: typeof fetch
): Promise<SingleRecipeSetResponse> {
    // ID: Mapping field ke skema Postman (yield_quantity)
    // EN: Map fields to Postman schema (yield_quantity)
    const mapped = {
        ...(data.name !== undefined ? { name: data.name } : {}),
        ...(data.yield_quantity !== undefined
            ? { yield_quantity: data.yield_quantity }
            : data.yield_qty !== undefined
            ? { yield_quantity: data.yield_qty }
            : {}),
        ...(data.effective_from !== undefined ? { effective_from: data.effective_from } : {}),
        ...(data.effective_to !== undefined ? { effective_to: data.effective_to } : {}),
        ...(data.is_active !== undefined ? { is_active: data.is_active } : {})
    };
    return makeRequest<SingleRecipeSetResponse>(
        `/api/recipe-sets/${uuid}`,
        {
            method: 'PUT',
            body: JSON.stringify(mapped)
        },
        cookies,
        fetchImpl
    );
}

// DELETE /api/recipe-sets/{uuid}
export async function deleteRecipeSetApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<DeleteRecipeSetResponse> {
	return makeRequest<DeleteRecipeSetResponse>(
		`/api/recipe-sets/${uuid}`,
		{
			method: 'DELETE'
		},
		cookies,
		fetchImpl
	);
}

// ========== RECIPE ITEM API FUNCTIONS ==========

// GET /api/recipe-items
export async function getRecipeItemsApi(
    params?: RecipeItemQueryParams,
    cookies?: string,
    fetchImpl?: typeof fetch
): Promise<RecipeItemResponse> {
    const searchParams = new URLSearchParams();

    if (params?.page) searchParams.set('page', params.page.toString());
    if (params?.limit) searchParams.set('limit', params.limit.toString());
    // ID: Gunakan nama filter Postman; fallback dari nama lama
    // EN: Use Postman filter names; fallback from legacy names
    const recipeFilter = params?.recipe_sets_uuid ?? params?.recipe_uuid;
    const ingredientFilter = params?.ingredient_stocks_uuid ?? params?.ingredient_uuid;
    if (recipeFilter) searchParams.set('recipe_sets_uuid', recipeFilter);
    if (ingredientFilter) searchParams.set('ingredient_stocks_uuid', ingredientFilter);

    const queryString = searchParams.toString();
    const endpoint = `/api/recipe-items${queryString ? `?${queryString}` : ''}`;

	return makeRequest<RecipeItemResponse>(
		endpoint,
		{
			method: 'GET'
		},
		cookies,
		fetchImpl
	);
}

// GET /api/recipe-items/{uuid}
export async function getRecipeItemApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleRecipeItemResponse> {
	return makeRequest<SingleRecipeItemResponse>(
		`/api/recipe-items/${uuid}`,
		{
			method: 'GET'
		},
		cookies,
		fetchImpl
	);
}

// POST /api/recipe-items
export async function createRecipeItemApi(
    data: CreateRecipeItemRequest,
    cookies?: string,
    fetchImpl?: typeof fetch
): Promise<SingleRecipeItemResponse> {
    // ID: Mapping body ke skema Postman (recipe_sets_uuid, ingredient_stocks_uuid, quantity, waste_percent)
    // EN: Map request body to Postman schema (recipe_sets_uuid, ingredient_stocks_uuid, quantity, waste_percent)
    const mapped = {
        recipe_sets_uuid: (data as any).recipe_sets_uuid ?? data.recipe_uuid,
        ingredient_stocks_uuid: (data as any).ingredient_stocks_uuid ?? data.ingredient_uuid,
        quantity: (data as any).quantity ?? data.qty,
        waste_percent: (data as any).waste_percent ?? data.waste_pct ?? 0
    };
    return makeRequest<SingleRecipeItemResponse>(
        '/api/recipe-items',
        {
            method: 'POST',
            body: JSON.stringify(mapped)
        },
        cookies,
        fetchImpl
    );
}

// PUT /api/recipe-items/{uuid}
export async function updateRecipeItemApi(
    uuid: string,
    data: UpdateRecipeItemRequest,
    cookies?: string,
    fetchImpl?: typeof fetch
): Promise<SingleRecipeItemResponse> {
    // ID: Mapping body ke skema Postman (quantity, waste_percent)
    // EN: Map request body to Postman schema (quantity, waste_percent)
    const mapped = {
        ...(data.quantity !== undefined
            ? { quantity: data.quantity }
            : data.qty !== undefined
            ? { quantity: data.qty }
            : {}),
        ...(data.waste_percent !== undefined
            ? { waste_percent: data.waste_percent }
            : data.waste_pct !== undefined
            ? { waste_percent: data.waste_pct }
            : {})
    };
    return makeRequest<SingleRecipeItemResponse>(
        `/api/recipe-items/${uuid}`,
        {
            method: 'PUT',
            body: JSON.stringify(mapped)
        },
        cookies,
        fetchImpl
    );
}

// DELETE /api/recipe-items/{uuid}
export async function deleteRecipeItemApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<DeleteRecipeItemResponse> {
	return makeRequest<DeleteRecipeItemResponse>(
		`/api/recipe-items/${uuid}`,
		{
			method: 'DELETE'
		},
		cookies,
		fetchImpl
	);
}

export { RecipeApiError };
