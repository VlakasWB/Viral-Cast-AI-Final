import { api, type ApiResponse } from '$lib/utils/api';
import type { IngredientCatalog, IngredientCatalogUom } from '$lib/types/ingredient-catalog.js';

const BASE_URL = '/api/v1/ingredient-catalog';

interface RawIngredientCatalog {
	[key: string]: unknown;
	uuid?: string;
	name?: string;
	description?: string | null;
	base_uom?: RawBaseUom | string;
	base_uom_uuid?: RawBaseUom | string;
	minimum_stock?: number | string | null;
	minimal_stock?: number | string | null;
	min_stock?: number | string | null;
	shelf_life_days?: number | string | null;
	created_at?: number | string | null;
	updated_at?: number | string | null;
	deleted_at?: number | string | null;
}

interface RawBaseUom {
	uuid?: string;
	code?: string;
	name?: string;
	symbol?: string;
	short_name?: string;
	label?: string;
}

interface RawPagination {
	page?: number | string;
	current_page?: number | string;
	limit?: number | string;
	per_page?: number | string;
	total?: number | string;
	total_pages?: number | string;
	last_page?: number | string;
	has_prev?: boolean;
	has_next?: boolean;
	prev_page?: number | string | null;
	next_page?: number | string | null;
}

type IngredientCatalogWritePayload = {
	name?: string;
	description?: string | null;
	base_uom?: IngredientCatalog['base_uom'] | string;
	base_uom_uuid?: string;
	minimum_stock?: number | string | null;
	minimal_stock?: number | string | null;
	min_stock?: number | string | null;
	shelf_life_days?: number | string | null;
};

const toStringSafe = (value: unknown): string => {
	if (value === null || value === undefined) return '';
	return String(value);
};

const toNumberSafe = (value: unknown, fallback = 0): number => {
	if (value === null || value === undefined) return fallback;
	if (typeof value === 'number') return Number.isFinite(value) ? value : fallback;
	if (typeof value === 'string') {
		const trimmed = value.trim();
		if (!trimmed) return fallback;
		const normalized = trimmed.replace(/[^\d.,-]/g, '').replace(',', '.');
		const parsed = Number(normalized);
		return Number.isFinite(parsed) ? parsed : fallback;
	}
	return fallback;
};

const normalizeTimestamp = (value: unknown): number => {
	if (value === null || value === undefined) {
		return Date.now();
	}

	if (typeof value === 'number') {
		return Number.isFinite(value) ? value : Date.now();
	}

	if (typeof value === 'string') {
		const numericValue = Number(value);
		if (Number.isFinite(numericValue)) {
			return numericValue;
		}
		const parsedDate = Date.parse(value);
		if (Number.isFinite(parsedDate)) {
			return parsedDate;
		}
	}

	return Date.now();
};

const resolveBaseUom = (raw: RawIngredientCatalog): IngredientCatalogUom => {
	const candidate = (raw.base_uom ?? raw.base_uom_uuid) as RawBaseUom | string | undefined;

	if (typeof candidate === 'string') {
		return {
			uuid: candidate,
			code: '',
			name: ''
		};
	}

	if (candidate && typeof candidate === 'object') {
		return {
			uuid: toStringSafe(candidate.uuid ?? raw.base_uom_uuid ?? ''),
			code: toStringSafe(candidate.code ?? candidate.symbol ?? candidate.short_name ?? ''),
			name: toStringSafe(candidate.name ?? candidate.label ?? '')
		};
	}

	return {
		uuid: toStringSafe(raw.base_uom_uuid ?? ''),
		code: '',
		name: ''
	};
};

const mapIngredientCatalog = (raw: RawIngredientCatalog): IngredientCatalog => {
	const deletedSource = raw.deleted_at ?? (raw as Record<string, unknown>).deletedAt;
	let deletedAt: number | null | undefined = undefined;

	if (deletedSource === null) {
		deletedAt = null;
	} else if (deletedSource !== undefined) {
		deletedAt = normalizeTimestamp(deletedSource);
	}

	const catalog: IngredientCatalog = {
		uuid: toStringSafe(raw.uuid ?? ''),
		name: toStringSafe(raw.name ?? ''),
		description: raw.description ?? undefined,
		base_uom: resolveBaseUom(raw),
		minimum_stock: toNumberSafe(
			raw.minimum_stock ?? raw.minimal_stock ?? raw.min_stock,
			0
		),
		shelf_life_days: toNumberSafe(raw.shelf_life_days, 0),
		created_at: normalizeTimestamp(raw.created_at),
		updated_at: normalizeTimestamp(raw.updated_at ?? raw.created_at ?? Date.now())
	};

	if (deletedAt !== undefined) {
		catalog.deleted_at = deletedAt;
	}

	return catalog;
};

const extractListPayload = (raw: unknown): { items: RawIngredientCatalog[]; pagination: RawPagination } => {
	const payload = ((raw ?? {}) as Record<string, unknown>) ?? {};

	const nested = payload['data'];
	const innerData =
		nested && typeof nested === 'object' && !Array.isArray(nested)
			? (nested as Record<string, unknown>)
			: payload;

	let itemsCandidate: unknown =
		innerData['ingredient_catalog'] ??
		innerData['ingredientCatalog'] ??
		innerData['items'];

	if (!Array.isArray(itemsCandidate)) {
		const deeper = innerData['data'];
		if (Array.isArray(deeper)) {
			itemsCandidate = deeper;
		} else if (deeper && typeof deeper === 'object') {
			const deeperRecord = deeper as Record<string, unknown>;
			itemsCandidate =
				deeperRecord['ingredient_catalog'] ??
				deeperRecord['ingredientCatalog'] ??
				deeperRecord['items'];
		}
	}

	if (!Array.isArray(itemsCandidate)) {
		itemsCandidate =
			payload['ingredient_catalog'] ??
			payload['ingredientCatalog'] ??
			payload['items'];
	}

	const items = Array.isArray(itemsCandidate)
		? (itemsCandidate as RawIngredientCatalog[])
		: [];

	const paginationSource =
		(innerData['pagination'] as RawPagination | undefined) ??
		(innerData['meta'] as RawPagination | undefined) ??
		(payload['pagination'] as RawPagination | undefined) ??
		(payload['meta'] as RawPagination | undefined) ??
		({});

	return {
		items,
		pagination: paginationSource ?? {}
	};
};

const buildRequestBody = (input: IngredientCatalogWritePayload): Record<string, unknown> => {
	const body: Record<string, unknown> = {};

	if (input.name !== undefined) body.name = input.name;
	if (input.description !== undefined) body.description = input.description;

	if (input.base_uom_uuid) {
		body.base_uom_uuid = input.base_uom_uuid;
	} else if (input.base_uom && typeof input.base_uom === 'object') {
		const candidate = input.base_uom as IngredientCatalogUom;
		if (candidate.uuid) {
			body.base_uom_uuid = candidate.uuid;
		}
	} else if (typeof input.base_uom === 'string' && input.base_uom) {
		body.base_uom_uuid = input.base_uom;
	}

	const minimumStockValue =
		input.minimum_stock ??
		input.minimal_stock ??
		input.min_stock ??
		(input as Record<string, unknown>)['minimumStock'] ??
		(input as Record<string, unknown>)['minimalStock'];
	if (
		minimumStockValue !== undefined &&
		minimumStockValue !== null &&
		minimumStockValue !== ''
	) {
		body.minimum_stock =
			typeof minimumStockValue === 'number'
				? minimumStockValue.toString()
				: String(minimumStockValue);
	}

	if (input.shelf_life_days !== undefined && input.shelf_life_days !== null) {
		body.shelf_life_days =
			typeof input.shelf_life_days === 'string'
				? Number.parseInt(input.shelf_life_days, 10)
				: input.shelf_life_days;
	}

	return body;
};

const unwrapSingle = (raw: unknown): RawIngredientCatalog => {
	if (!raw || typeof raw !== 'object') {
		return {};
	}

	if ('data' in raw && raw.data && typeof raw.data === 'object') {
		return unwrapSingle(raw.data);
	}

	return raw as RawIngredientCatalog;
};

/**
 * Mendapatkan daftar ingredient catalog dengan pagination dan pencarian.
 * ID: Mengambil daftar katalog bahan dengan paginasi dan pencarian.
 * EN: Get ingredient catalog list with pagination and search.
 */
export async function getIngredientCatalogsApi(
	page = 1,
	limit = 10,
	search = '',
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<{
	items: IngredientCatalog[];
	total: number;
	page: number;
	pageCount: number;
	limit: number;
	hasNext: boolean;
	hasPrev: boolean;
}> {
	const params = new URLSearchParams();
	params.set('page', String(page));
	params.set('limit', String(limit));
	if (search) params.set('search', search);

	const endpoint = params.toString() ? `${BASE_URL}?${params.toString()}` : BASE_URL;
	const response = await api.get<ApiResponse<unknown>>(endpoint, cookies, fetchImpl);
	const { items: rawItems, pagination } = extractListPayload(response.data);

	const currentPage = toNumberSafe(
		pagination.page ?? pagination.current_page,
		page
	) || page;
	const perPage = toNumberSafe(
		pagination.limit ?? pagination.per_page,
		limit
	) || limit;
	const total = toNumberSafe(pagination.total, rawItems.length);
	const totalPages =
		toNumberSafe(
			pagination.total_pages ?? pagination.last_page,
			perPage > 0 ? Math.max(1, Math.ceil(total / perPage)) : 1
		) || 1;

	const hasPrev =
		typeof pagination.has_prev === 'boolean'
			? pagination.has_prev
			: currentPage > 1;
	const hasNext =
		typeof pagination.has_next === 'boolean'
			? pagination.has_next
			: currentPage < totalPages;

	return {
		items: rawItems.map(mapIngredientCatalog),
		total,
		page: currentPage,
		pageCount: totalPages,
		limit: perPage,
		hasNext,
		hasPrev
	};
}

/**
 * Mendapatkan detail ingredient catalog berdasarkan UUID.
 * ID: Mengambil detail katalog bahan berdasarkan UUID.
 * EN: Get ingredient catalog detail by UUID.
 */
export async function getIngredientCatalogApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<IngredientCatalog> {
	const encodedUuid = encodeURIComponent(uuid);
	const response = await api.get<ApiResponse<unknown>>(
		`${BASE_URL}/${encodedUuid}`,
		cookies,
		fetchImpl
	);
	const raw = unwrapSingle(response.data);
	return mapIngredientCatalog(raw);
}

/**
 * Membuat ingredient catalog baru.
 * ID: Membuat katalog bahan baru.
 * EN: Create new ingredient catalog.
 */
export async function createIngredientCatalogApi(
	data: IngredientCatalogWritePayload,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<IngredientCatalog> {
	const body = buildRequestBody(data);
	const response = await api.post<ApiResponse<unknown>>(
		BASE_URL,
		body,
		cookies,
		fetchImpl
	);
	const raw = unwrapSingle(response.data);
	return mapIngredientCatalog(raw);
}

/**
 * Memperbarui ingredient catalog.
 * ID: Memperbarui katalog bahan.
 * EN: Update ingredient catalog.
 */
export async function updateIngredientCatalogApi(
	uuid: string,
	data: IngredientCatalogWritePayload,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<IngredientCatalog> {
	const body = buildRequestBody(data);
	const encodedUuid = encodeURIComponent(uuid);
	const response = await api.put<ApiResponse<unknown>>(
		`${BASE_URL}/${encodedUuid}`,
		body,
		cookies,
		fetchImpl
	);
	const raw = unwrapSingle(response.data);
	return mapIngredientCatalog(raw);
}

/**
 * Menghapus ingredient catalog.
 * ID: Menghapus katalog bahan.
 * EN: Delete ingredient catalog.
 */
export async function deleteIngredientCatalogApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<void> {
	const encodedUuid = encodeURIComponent(uuid);
	await api.delete(`${BASE_URL}/${encodedUuid}`, cookies, fetchImpl);
}
