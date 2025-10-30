import { api, type ApiResponse } from '$lib/utils/api';
import type { IngredientStock, IngredientStockResponse } from '$lib/types/ingredientStock.js';
import type { Ingredient } from '$lib/types/ingredient.js';

const BASE_URL = '/api/v1/ingredient-stocks';

const toNumber = (value: unknown, fallback = 0): number => {
	if (value === null || value === undefined) return fallback;
	if (typeof value === 'number') return Number.isFinite(value) ? value : fallback;
	if (typeof value === 'string' && value.trim() !== '') {
		const parsed = Number(value);
		return Number.isFinite(parsed) ? parsed : fallback;
	}
	return fallback;
};

const toString = (value: unknown, fallback = ''): string =>
	value === null || value === undefined ? fallback : String(value);

const normalizeTimestamp = (value: unknown): number => {
	const numeric = toNumber(value, Number.NaN);
	if (!Number.isNaN(numeric)) return numeric;
	if (typeof value === 'string') {
		const parsed = Date.parse(value);
		if (!Number.isNaN(parsed)) return parsed;
	}
	return Date.now();
};

const buildIngredient = (raw: any): Ingredient => {
	const ingredientSource = raw?.ingredient ?? raw?.ingredient_catalog ?? {};
	const baseUomSource = ingredientSource?.base_uom ?? ingredientSource?.baseUnit ?? {};

	return {
		uuid: toString(
			ingredientSource?.uuid ?? raw?.ingredient_catalog_uuid ?? raw?.ingredient_uuid ?? ''
		),
		name: toString(
			ingredientSource?.name ?? raw?.ingredient_name ?? raw?.name ?? 'Unknown Ingredient'
		),
		base_uom: {
			uuid: toString(baseUomSource?.uuid ?? ''),
			code: toString(baseUomSource?.code ?? baseUomSource?.symbol ?? ''),
			name: toString(baseUomSource?.name ?? '')
		},
		min_stock: toString(ingredientSource?.min_stock ?? ingredientSource?.minimum_stock ?? '0'),
		shelf_life_days: toNumber(ingredientSource?.shelf_life_days, 0),
		created_at: normalizeTimestamp(ingredientSource?.created_at ?? raw?.created_at),
		updated_at: normalizeTimestamp(ingredientSource?.updated_at ?? raw?.updated_at)
	};
};

const normalizeStock = (raw: any): IngredientStock => {
	const ingredient = buildIngredient(raw);
	const unitOfMeasureCode = toString(
		raw?.unit_of_measure_code ??
			raw?.unit_code ??
			raw?.unit ??
			raw?.uom ??
			raw?.unit_of_measure?.code ??
			raw?.unit_of_measure?.symbol ??
			ingredient.base_uom?.code ??
			''
	).trim();
	const unitOfMeasureName = toString(
		raw?.unit_of_measure_name ??
			raw?.unit_name ??
			raw?.unit_of_measure?.name ??
			ingredient.base_uom?.name ??
			''
	).trim();

	return {
		uuid: toString(raw?.uuid),
		ingredient_uuid: ingredient.uuid,
		ingredient,
		total_quantity: toNumber(raw?.total_quantity ?? raw?.quantity),
		unit_of_measure_code: unitOfMeasureCode || undefined,
		unit_of_measure_name: unitOfMeasureName || undefined,
		total_value: toNumber(raw?.total_value ?? raw?.value),
		current_cost: toNumber(raw?.current_cost ?? raw?.current_price),
		avg_cost: toNumber(raw?.avg_cost ?? raw?.average_cost),
		created_at: normalizeTimestamp(raw?.created_at),
		updated_at: normalizeTimestamp(raw?.updated_at ?? raw?.created_at),
		ingredient_stock_move_uuid: raw?.ingredient_stock_move_uuid
			? toString(raw.ingredient_stock_move_uuid)
			: undefined,
		ingredient_catalog_uuid: ingredient.uuid,
		ingredient_name: ingredient.name
	};
};

const unwrapData = <T>(resp: ApiResponse<T> | T): T => {
	if (resp && typeof resp === 'object' && 'data' in resp) {
		return (resp as ApiResponse<T>).data;
	}
	return resp as T;
};

const extractList = (raw: unknown): IngredientStock[] => {
	if (!raw || typeof raw !== 'object') return [];
	if (Array.isArray(raw)) return raw.map(normalizeStock);

	const record = raw as Record<string, unknown>;

	const rows =
		record.data && Array.isArray(record.data)
			? record.data
			: record.items && Array.isArray(record.items)
				? record.items
				: Array.isArray(record.ingredient_stocks)
					? record.ingredient_stocks
					: [];

	return (rows as unknown[]).map(normalizeStock);
};

const extractMeta = (
	raw: any,
	fallback: { page?: number; limit?: number; total?: number }
): { page: number; limit: number; total: number; pageCount: number } => {
	const limit = toNumber(
		raw?.limit ?? raw?.per_page ?? raw?.page_size ?? fallback.limit ?? 10,
		fallback.limit ?? 10
	);
	const page = toNumber(raw?.page ?? raw?.current_page ?? fallback.page ?? 1, fallback.page ?? 1);
	const total = toNumber(raw?.total ?? raw?.count ?? fallback.total ?? 0, fallback.total ?? 0);
	const pageCount =
		toNumber(raw?.total_pages ?? raw?.last_page, Number.NaN) ||
		Math.max(1, Math.ceil(total / Math.max(limit, 1)));

	return {
		page: page || 1,
		limit: limit || 10,
		total,
		pageCount
	};
};

/**
 * Mendapatkan daftar stok bahan dengan pagination
 */
export async function getIngredientStocksApi(
	page = 1,
	size = 10,
	search = '',
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<IngredientStockResponse> {
	const params = new URLSearchParams();
	params.set('page', String(page));
	params.set('limit', String(size));
	if (search) {
		params.set('search', search);
	}

	const endpoint = params.size ? `${BASE_URL}?${params.toString()}` : BASE_URL;
	const resp = await api.get<unknown>(endpoint, cookies, fetchImpl);
	const payload = unwrapData(resp) ?? {};
	const items = extractList(payload ?? resp);
	const metaSource =
		(payload as any)?.meta ??
		(payload as any)?.pagination ??
		(resp as any)?.meta ??
		(resp as any)?.pagination ??
		{};

	const meta = extractMeta(metaSource, { page, limit: size, total: items.length });

	return {
		data: items,
		meta: {
			total: meta.total,
			current_page: meta.page,
			last_page: meta.pageCount,
			per_page: meta.limit
		},
		page: meta.page,
		pageCount: meta.pageCount,
		limit: meta.limit
	};
}

/**
 * Mendapatkan detail stok bahan berdasarkan UUID
 */
export async function getIngredientStockByIdApi(
	id: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<IngredientStock> {
	const resp = await api.get<unknown>(`${BASE_URL}/${id}`, cookies, fetchImpl);
	return normalizeStock(unwrapData(resp) ?? resp);
}

/**
 * Membuat stok bahan baru
 */
export async function createIngredientStockApi(
	stockData: Record<string, unknown>,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<IngredientStock> {
	const resp = await api.post<unknown>(BASE_URL, stockData, cookies, fetchImpl);
	return normalizeStock(unwrapData(resp) ?? resp);
}

/**
 * Memperbarui stok bahan yang ada
 */
export async function updateIngredientStockApi(
	id: string,
	stockData: Record<string, unknown>,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<IngredientStock> {
	const resp = await api.put<unknown>(`${BASE_URL}/${id}`, stockData, cookies, fetchImpl);
	return normalizeStock(unwrapData(resp) ?? resp);
}

/**
 * Menghapus stok bahan
 */
export async function deleteIngredientStockApi(
	id: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<boolean> {
	await api.delete<unknown>(`${BASE_URL}/${id}`, cookies, fetchImpl);
	return true;
}
