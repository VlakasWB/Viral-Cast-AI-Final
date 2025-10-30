import type {
	UOM,
	CreateUOMRequest,
	UpdateUOMRequest,
	UOMResponse,
	SingleUOMResponse,
	DeleteUOMResponse,
	PaginationMeta
} from '$lib/types/uom';
import { browser } from '$app/environment';

// Ambil base URL API (server & client) tanpa $env/static
const getApiBaseUrl = () => {
  try {
    return process.env.API_BASE_URL || 'http://localhost:12000';
  } catch {
    return 'http://localhost:12000';
  }
};

const DEFAULT_PAGINATION: PaginationMeta = {
	total: 0,
	last_page: 1,
	current_page: 1,
	per_page: 10,
	from: 0,
	to: 0
};

const toNumber = (value: unknown, fallback: number): number => {
	if (value === null || value === undefined) {
		return fallback;
	}
	const num = Number(value);
	return Number.isFinite(num) ? num : fallback;
};

const normalizeUOMArray = (payload: any): UOM[] => {
	if (Array.isArray(payload)) return payload;
	if (Array.isArray(payload?.data)) return payload.data;
	if (Array.isArray(payload?.items)) return payload.items;
	if (Array.isArray(payload?.rows)) return payload.rows;
	return [];
};

const normalizePaginationMeta = (
	raw: any,
	fallback: { page?: number; limit?: number; total?: number; chunkSize?: number }
): PaginationMeta => {
	if (!raw) raw = {};

	const limitFallback = fallback.limit ?? fallback.chunkSize ?? DEFAULT_PAGINATION.per_page;
	const totalFallback = fallback.total ?? DEFAULT_PAGINATION.total;
	const pageFallback = fallback.page ?? DEFAULT_PAGINATION.current_page;

	let perPage = toNumber(
		raw.per_page ?? raw.limit ?? raw.page_size ?? raw.pageSize ?? raw.perPage,
		limitFallback
	);
	if (perPage <= 0) perPage = limitFallback > 0 ? limitFallback : DEFAULT_PAGINATION.per_page;

	const total = toNumber(
		raw.total ?? raw.count ?? raw.total_items ?? raw.totalItems ?? raw.totalCount,
		totalFallback
	);

	let currentPage = toNumber(
		raw.current_page ?? raw.page ?? raw.currentPage ?? raw.page_index ?? raw.pageIndex,
		pageFallback
	);
	if (currentPage <= 0) currentPage = pageFallback > 0 ? pageFallback : DEFAULT_PAGINATION.current_page;

	let lastPage = toNumber(
		raw.last_page ?? raw.total_pages ?? raw.page_count ?? raw.pageCount ?? raw.totalPages,
		Math.max(1, Math.ceil(total / (perPage || 1)))
	);
	if (lastPage <= 0) {
		lastPage = Math.max(1, Math.ceil(total / (perPage || 1)));
	}

	const computedFrom = (currentPage - 1) * perPage + (total > 0 ? 1 : 0);
	const from = toNumber(raw.from, computedFrom);
	const computedTo = Math.min(currentPage * perPage, total);
	const to = toNumber(raw.to, computedTo < from ? from + perPage - 1 : computedTo);

	return {
		total,
		last_page: lastPage,
		current_page: currentPage,
		per_page: perPage,
		from: Math.max(0, from),
		to: Math.max(0, to)
	};
};

const normalizeUOMResponse = (
	raw: any,
	fallback: { page?: number; limit?: number }
): UOMResponse => {
	const items = normalizeUOMArray(raw?.data ?? []);
	const metaSource =
		raw?.meta ?? raw?.data?.meta ?? raw?.data?.pagination ?? raw?.pagination ?? undefined;
	const meta = normalizePaginationMeta(metaSource, {
		page: fallback.page,
		limit: fallback.limit,
		total: Array.isArray(items) ? items.length : 0,
		chunkSize: Array.isArray(items) ? items.length : undefined
	});

	return {
		code: toNumber(raw?.code, 200),
		status: raw?.status ?? 'OK',
		message: raw?.message ?? '',
		data: items,
		meta,
		errors: raw?.errors ?? {}
	};
};

class UOMApiError extends Error {
	constructor(
		message: string,
		public status: number,
		public response?: any
	) {
		super(message);
		this.name = 'UOMApiError';
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
			throw new UOMApiError(
				errorData.message || `HTTP ${response.status}: ${response.statusText}`,
				response.status,
				errorData
			);
		}

		return await response.json();
	} catch (error) {
		if (error instanceof UOMApiError) {
			throw error;
		}
		throw new UOMApiError(error instanceof Error ? error.message : 'Network error occurred', 0);
	}
}

// GET /api/v1/uoms
export async function getUOMsApi(
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<UOMResponse> {
	const raw = await makeRequest<any>(
		'/api/v1/uoms',
		{
			method: 'GET'
		},
		cookies,
		fetchImpl
	);

	return normalizeUOMResponse(raw, { page: 1 });
}

// GET /api/v1/uoms/{uuid}
export async function getUOMApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleUOMResponse> {
	return makeRequest<SingleUOMResponse>(
		`/api/v1/uoms/${uuid}`,
		{
			method: 'GET'
		},
		cookies,
		fetchImpl
	);
}

// POST /api/v1/uoms
export async function createUOMApi(
	data: CreateUOMRequest,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleUOMResponse> {
	return makeRequest<SingleUOMResponse>(
		'/api/v1/uoms',
		{
			method: 'POST',
			body: JSON.stringify(data)
		},
		cookies,
		fetchImpl
	);
}

// PATCH /api/v1/uoms/{uuid}
export async function updateUOMApi(
	uuid: string,
	data: UpdateUOMRequest,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<SingleUOMResponse> {
	return makeRequest<SingleUOMResponse>(
		`/api/v1/uoms/${uuid}`,
		{
			method: 'PATCH',
			body: JSON.stringify(data)
		},
		cookies,
		fetchImpl
	);
}

// DELETE /api/v1/uoms/{uuid}
export async function deleteUOMApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<DeleteUOMResponse> {
	return makeRequest<DeleteUOMResponse>(
		`/api/v1/uoms/${uuid}`,
		{
			method: 'DELETE'
		},
		cookies,
		fetchImpl
	);
}

export { UOMApiError };

// Search UOMs - uses GET /api/v1/uoms with query parameters
export async function searchUnitsOfMeasureApi(
	params?: { page?: number; limit?: number; search?: string; code?: string; name?: string },
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<UOMResponse> {
	const searchParams = new URLSearchParams();
	if (params?.page !== undefined) searchParams.set('page', params.page.toString());
	if (params?.limit !== undefined) searchParams.set('limit', params.limit.toString());
	if (params?.search) searchParams.set('search', params.search);
	if (params?.code) searchParams.set('code', params.code);
	if (params?.name) searchParams.set('name', params.name);

	const query = searchParams.toString();
	const endpoint = query ? `/api/v1/uoms?${query}` : '/api/v1/uoms';

	const raw = await makeRequest<any>(
		endpoint,
		{ method: 'GET' },
		cookies,
		fetchImpl
	);

	return normalizeUOMResponse(raw, {
		page: params?.page,
		limit: params?.limit
	});
}

