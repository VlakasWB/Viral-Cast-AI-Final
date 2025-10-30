import { redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import {
	getIngredientStockMovesApi,
	deleteIngredientStockMoveApi
} from '$lib/services/ingredient-stock-moves';
import { getIngredientCatalogsApi } from '$lib/services/ingredient-catalog';
import type {
	IngredientStockMove,
	IngredientStockMoveType
} from '$lib/types/ingredient-stock-moves';
import type { IngredientCatalog } from '$lib/types/ingredient-catalog';

const normalizeDateParam = (value: string | null): string | undefined => {
	if (!value) return undefined;
	const trimmed = value.trim();
	if (!trimmed) return undefined;

	// Allow timestamp in milliseconds (10+ digits)
	if (/^\d{10,}$/.test(trimmed)) {
		return trimmed;
	}

	const parsed = Date.parse(trimmed);
	if (Number.isNaN(parsed)) {
		return undefined;
	}

	return parsed.toString();
};

const loadAllIngredientCatalogs = async (
	cookieStr: string,
	fetchImpl: typeof fetch
): Promise<IngredientCatalog[]> => {
	const aggregated: IngredientCatalog[] = [];
	let page = 1;
	const pageSize = 100;

	while (true) {
		const response = await getIngredientCatalogsApi(page, pageSize, '', cookieStr, fetchImpl);
		aggregated.push(...response.items);

		if (!response.hasNext || page >= response.pageCount) {
			break;
		}

		if (!response.items.length) {
			break;
		}

		page += 1;
	}

	return aggregated;
};

// ID: Fungsi untuk memuat data ingredient stock moves
// EN: Function to load ingredient stock moves data
export const load: PageServerLoad = async ({ url, locals, cookies, fetch }) => {
    // ID: Pastikan user sudah login
    // EN: Ensure user is logged in
    const session = await locals.getSession();
    if (!session) {
        throw redirect(302, '/login');
    }

    // ID: Ambil parameter query
    // EN: Get query parameters
    const page = Number(url.searchParams.get('page') || '1');
    const size = Number(url.searchParams.get('size') || '10');
    const search = url.searchParams.get('search') || '';
    const ingredientName = url.searchParams.get('name') || '';
    const move_type = url.searchParams.get('move_type') || '';
	const ref_uuid = url.searchParams.get('ref_uuid') || '';
	const from_date = url.searchParams.get('from_date') || '';
	const to_date = url.searchParams.get('to_date') || '';
	const sort_by = url.searchParams.get('sort_by') || '';
	const sort_direction_param = url.searchParams.get('sort_direction');
	const sort_direction = sort_direction_param === 'asc' ? 'asc' : sort_direction_param === 'desc' ? 'desc' : undefined;

    try {
        // ID: Ambil data dari API
        // EN: Fetch data from API
		const normalizedMoveType = move_type
			? (move_type.toUpperCase() as IngredientStockMoveType)
			: undefined;
		const normalizedFromDate = normalizeDateParam(from_date);
		const normalizedToDate = normalizeDateParam(to_date);

		const filters = {
			limit: size,
			offset: (page - 1) * size,
			search: search || undefined,
			name: ingredientName || undefined,
			ingredient: ingredientName || undefined,
			ref_type: normalizedMoveType,
			ref_uuid: ref_uuid || undefined,
			from_date: normalizedFromDate,
			to_date: normalizedToDate,
			page,
			sort_by: sort_by || undefined,
			sort_direction
		};

		const accessToken = cookies.get('access_token');
		if (!accessToken) {
			throw redirect(302, '/login');
		}
		const cookieStr = `access_token=${accessToken}`;
		const [stockMovesResult, ingredientsResult] = await Promise.allSettled([
			getIngredientStockMovesApi(filters, cookieStr, fetch),
			loadAllIngredientCatalogs(cookieStr, fetch) // ID: Ambil semua ingredients untuk filter / EN: Get all ingredients for filter
		]);

		if (stockMovesResult.status !== 'fulfilled') {
			console.error('Failed to fetch ingredient stock moves:', stockMovesResult.reason);
			throw stockMovesResult.reason;
		}

		const stockMovesData = stockMovesResult.value;
		const ingredientsData =
			ingredientsResult.status === 'fulfilled' ? ingredientsResult.value ?? [] : [];

        return {
			stockMoves: stockMovesData.data,
			total: stockMovesData.total,
			page,
			pageCount: stockMovesData.pageCount ?? Math.ceil(stockMovesData.total / size),
            search,
            size,
            name: ingredientName,
            move_type,
			ref_uuid,
			from_date,
			to_date,
			sort_by,
			sort_direction: sort_direction ?? '',
            ingredients: ingredientsData
        };
    } catch (error) {
        console.error('Error loading ingredient stock moves:', error);
        return {
            stockMoves: [],
            total: 0,
            page: 1,
            pageCount: 1,
           search,
           size,
            name: ingredientName,
            move_type,
			ref_uuid,
			from_date,
			to_date,
			sort_by,
			sort_direction: sort_direction ?? '',
           ingredients: []
        };
    }
};

// ID: Definisi actions untuk operasi CRUD
// EN: Define actions for CRUD operations
export const actions: Actions = {
    // ID: Action untuk menghapus ingredient stock move
    // EN: Action to delete ingredient stock move
	delete: async ({ request, locals, cookies }) => {
        // ID: Pastikan user sudah login
        // EN: Ensure user is logged in
        const session = await locals.getSession();
        if (!session) {
            throw redirect(302, '/login');
        }

        const formData = await request.formData();
        const uuid = formData.get('uuid')?.toString();

        if (!uuid) {
            return fail(400, { error: 'UUID is required' });
        }

        try {
			const accessToken = cookies.get('access_token');
			if (!accessToken) {
				throw redirect(302, '/login');
			}
			const cookieStr = `access_token=${accessToken}`;
			await deleteIngredientStockMoveApi(uuid, cookieStr);
            return { success: true };
        } catch (error) {
            console.error('Error deleting ingredient stock move:', error);
            return fail(500, { error: 'Failed to delete ingredient stock move' });
        }
    }
};
