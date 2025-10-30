import { api, type ApiResponse } from '$lib/utils/api';
import type {
	CreateIngredientStockMoveRequest,
	IngredientStockMove,
	IngredientStockMoveFilters,
	IngredientStockMovesResponse,
	UpdateIngredientStockMoveRequest
} from '$lib/types/ingredient-stock-moves.js';

const BASE_URL = '/api/v1/ingredient-stock-moves';

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

const toNullableNumber = (value: unknown): number | null => {
	if (value === null || value === undefined || value === '') return null;
	const parsed = toNumber(value, Number.NaN);
	return Number.isNaN(parsed) ? null : parsed;
};

const normalizeTimestamp = (value: unknown): number => {
	const numeric = toNumber(value, Number.NaN);
	if (!Number.isNaN(numeric)) return numeric;
	if (typeof value === 'string') {
		const parsed = Date.parse(value);
		if (!Number.isNaN(parsed)) return parsed;
	}
	return Date.now();
};

const toIsoString = (value: unknown): string => new Date(normalizeTimestamp(value)).toISOString();

const normalizeMove = (raw: any): IngredientStockMove => {
	const ingredientCatalogUuid = toString(raw?.ingredient_catalog_uuid ?? raw?.ingredient_uuid);
	const moveType = toString(raw?.ref_type ?? raw?.move_type ?? 'ADJUSTMENT').toUpperCase();
	const ingredientName = toString(
		raw?.ingredient_catalog?.name ?? raw?.ingredient_name ?? raw?.name ?? ''
	);
	const baseUomSource = raw?.ingredient_catalog?.base_uom ?? raw?.ingredient_catalog?.baseUnit;
	const unitOfMeasureSource = raw?.unit_of_measure ?? raw?.uom ?? {};
	const unitOfMeasureCode = toString(
		raw?.unit_of_measure_code ??
			unitOfMeasureSource?.code ??
			unitOfMeasureSource?.symbol ??
			baseUomSource?.code ??
			baseUomSource?.symbol ??
			raw?.unit ?? ''
	).trim();
	const unitOfMeasureName = toString(
		raw?.unit_of_measure_name ??
			unitOfMeasureSource?.name ??
			baseUomSource?.name ??
			raw?.unit_name ??
			raw?.unit ??
			''
	).trim();
	const storeName = toString(raw?.store?.name ?? raw?.store_name ?? '');
	const storeUuid = toString(raw?.store?.uuid ?? raw?.store_uuid ?? '');

	return {
		uuid: toString(raw?.uuid),
		ingredient_uuid: ingredientCatalogUuid,
		store_uuid: storeUuid || undefined,
		quantity: toNumber(raw?.quantity ?? raw?.qty ?? raw?.amount),
		unit_of_measure_code: unitOfMeasureCode || undefined,
		unit_of_measure_name: unitOfMeasureName || undefined,
		unit_of_measure_uuid: raw?.unit_of_measure_uuid
			? toString(raw.unit_of_measure_uuid)
			: undefined,
		move_type: moveType,
		reference: raw?.reference ? toString(raw.reference) : undefined,
		created_at: toIsoString(raw?.created_at),
		updated_at: toIsoString(raw?.updated_at ?? raw?.created_at),
		ingredient:
			(raw?.ingredient_catalog && Object.keys(raw.ingredient_catalog).length > 0) ||
			raw?.ingredient_name ||
			raw?.name
				? {
						uuid: ingredientCatalogUuid,
						name: ingredientName,
						base_uom: baseUomSource
							? toString(baseUomSource.code ?? baseUomSource.symbol ?? baseUomSource.name ?? '')
							: undefined
					}
				: undefined,
		store: storeName
			? {
					uuid: storeUuid || undefined,
					name: storeName
				}
			: undefined,
		unit_of_measure: raw?.unit_of_measure
			? {
					uuid: toString(raw.unit_of_measure.uuid),
					name: toString(raw.unit_of_measure.name),
					symbol: raw.unit_of_measure.symbol
						? toString(raw.unit_of_measure.symbol)
						: undefined,
					code: unitOfMeasureCode || undefined
				}
			: undefined,
		ingredient_catalog_uuid: ingredientCatalogUuid,
		price: toNumber(raw?.price ?? raw?.unit_price),
		price_updated_at: toNullableNumber(raw?.price_updated_at),
		effective_at: toNullableNumber(raw?.effective_at),
		expiry_at: toNullableNumber(raw?.expiry_at),
		ref_type: moveType,
		ref_uuid: raw?.ref_uuid ? toString(raw.ref_uuid) : null,
		name: raw?.name ? toString(raw.name) : undefined
	};
};

const extractListPayload = (raw: unknown): IngredientStockMove[] => {
	if (!raw || typeof raw !== 'object') return [];
	if (Array.isArray(raw)) return raw.map(normalizeMove);

	const record = raw as Record<string, unknown>;

	// Try common collection shapes
	let collectionCandidate: unknown[] | undefined;

	if (Array.isArray(record.data)) {
		collectionCandidate = record.data as unknown[];
	} else if (Array.isArray(record.items)) {
		collectionCandidate = record.items as unknown[];
	} else if (Array.isArray(record.ingredient_stock_moves)) {
		collectionCandidate = record.ingredient_stock_moves as unknown[];
	} else if (record.data && typeof record.data === 'object') {
		const nested = record.data as Record<string, unknown>;
		if (Array.isArray(nested.items)) {
			collectionCandidate = nested.items as unknown[];
		} else if (Array.isArray(nested.data)) {
			collectionCandidate = nested.data as unknown[];
		}
	}

	if (!collectionCandidate) return [];
	return collectionCandidate.map(normalizeMove);
};

const extractMeta = (
	raw: any,
	fallback: { limit?: number; page?: number; total?: number; offset?: number }
) => {
	const limit = toNumber(
		raw?.limit ?? raw?.per_page ?? raw?.page_size ?? fallback.limit ?? 10,
		fallback.limit ?? 10
	);
	let page = toNumber(raw?.page ?? raw?.current_page ?? fallback.page ?? 1, fallback.page ?? 1);
	if (!page && typeof fallback.offset === 'number') {
		page = Math.floor(fallback.offset / Math.max(limit, 1)) + 1;
	}
	const total = toNumber(raw?.total ?? raw?.count ?? fallback.total ?? 0, fallback.total ?? 0);
	const pageCount =
		toNumber(raw?.total_pages ?? raw?.last_page, Number.NaN) ??
		Math.max(1, Math.ceil(total / (limit || 1)));

	return {
		page: page || 1,
		limit: limit || 10,
		total,
		pageCount: pageCount || Math.max(1, Math.ceil(total / Math.max(limit, 1)))
	};
};

const unwrapData = <T>(resp: ApiResponse<T> | T): T => {
	if (resp && typeof resp === 'object' && 'data' in resp) {
		return (resp as ApiResponse<T>).data;
	}
	return resp as T;
};

export async function getIngredientStockMovesApi(
	filters?: IngredientStockMoveFilters,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<IngredientStockMovesResponse> {
	const params = new URLSearchParams();
	if (filters?.page) params.set('page', filters.page.toString());
	if (filters?.limit) params.set('limit', filters.limit.toString());
	const searchValue = filters?.search?.toString().trim();
	if (searchValue) {
		params.set('search', searchValue);
	}

	const rawNameFilter =
		typeof filters?.name === 'string' ? filters.name : undefined;
	const ingredientNameFilter =
		typeof rawNameFilter === 'string' && rawNameFilter.trim()
			? rawNameFilter.trim()
			: typeof filters?.ingredient === 'string'
				? filters.ingredient.trim()
				: filters?.ingredient;
	if (ingredientNameFilter) {
		params.set('name', ingredientNameFilter.toString());
	} else {
		const ingredientUuidFilter =
			filters?.ingredient_catalog_uuid ?? filters?.ingredient_uuid ?? undefined;
		if (ingredientUuidFilter) {
			const normalized = ingredientUuidFilter.toString().trim();
			if (normalized) {
				params.set('ingredient_catalog_uuid', normalized);
				params.set('ingredient_uuid', normalized);
			}
		}
	}

	const moveFilter = filters?.ref_type ?? filters?.move_type;
	if (moveFilter) params.set('ref_type', moveFilter.toString());

	if (filters?.sort_by) params.set('sort_by', filters.sort_by.toString());
	const sortDirection = filters?.sort_direction?.toLowerCase();
	if (sortDirection === 'asc' || sortDirection === 'desc') {
		params.set('sort_direction', sortDirection);
	}

	if (filters?.store_uuid) params.set('store_uuid', filters.store_uuid);
	if (filters?.ref_uuid) params.set('ref_uuid', filters.ref_uuid);
	if (filters?.from_date) params.set('from_date', filters.from_date);
	if (filters?.to_date) params.set('to_date', filters.to_date);
	if (typeof filters?.offset === 'number') params.set('offset', filters.offset.toString());

	const query = params.toString();
	const endpoint = query ? `${BASE_URL}?${query}` : BASE_URL;
	const resp = await api.get<unknown>(endpoint, cookies, fetchImpl);
	const payload = unwrapData(resp) ?? {};

	const items = extractListPayload(payload ?? resp);
	const metaSource =
		(payload as any)?.meta ??
		(payload as any)?.pagination ??
		(payload as any)?.data?.meta ??
		(payload as any)?.data?.pagination ??
		(resp as any)?.meta ??
		(resp as any)?.pagination ??
		{};

	const meta = extractMeta(metaSource, {
		page: filters?.page,
		limit: filters?.limit,
		offset: filters?.offset,
		total: Array.isArray(items) ? items.length : 0
	});

	return {
		data: items,
		total: meta.total,
		limit: meta.limit,
		offset: (meta.page - 1) * meta.limit,
		page: meta.page,
		pageCount: meta.pageCount
	};
}

export async function getIngredientStockMoveApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<IngredientStockMove> {
	const resp = await api.get<unknown>(`${BASE_URL}/${uuid}`, cookies, fetchImpl);
	const payload = unwrapData(resp);
	if (Array.isArray(payload)) {
		return normalizeMove(payload[0] ?? {});
	}
	return normalizeMove(payload ?? resp);
}

export async function createIngredientStockMoveApi(
	data: CreateIngredientStockMoveRequest,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<IngredientStockMove> {
	const resp = await api.post<unknown>(BASE_URL, data, cookies, fetchImpl);
	return normalizeMove(unwrapData(resp) ?? resp);
}

export async function updateIngredientStockMoveApi(
	uuid: string,
	data: UpdateIngredientStockMoveRequest,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<IngredientStockMove> {
	const resp = await api.patch<unknown>(`${BASE_URL}/${uuid}`, data, cookies, fetchImpl);
	return normalizeMove(unwrapData(resp) ?? resp);
}

export async function deleteIngredientStockMoveApi(
	uuid: string,
	cookies?: string,
	fetchImpl?: typeof fetch
): Promise<void> {
	await api.delete(`${BASE_URL}/${uuid}`, cookies, fetchImpl);
}
