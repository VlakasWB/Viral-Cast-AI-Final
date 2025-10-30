import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { getIngredientStocksApi, deleteIngredientStockApi } from '$lib/services/ingredientStock';
import { getStoreIngredientPredictionsApi } from '$lib/services/storeIngredientPredictions';
import type { IngredientStock } from '$lib/types/ingredientStock';
import type { StoreIngredientPredictionsData } from '$lib/types/store-ingredient-prediction';
import { ApiError } from '$lib/utils/api';

// Dummy data for development (will be removed when API is fully integrated)
let dummyIngredientStocks: IngredientStock[] = [
	{
		uuid: '58a83262-9796-4667-b86c-bd128a83ba3b',
		ingredient_uuid: '123e4567-e89b-12d3-a456-426614174000',
		ingredient: {
			uuid: '123e4567-e89b-12d3-a456-426614174000',
			name: 'Pure Vanilla Extract',
			base_uom: {
				uuid: '2198a07b-f9a0-4924-99f6-85ef4fc78178',
				code: 'L',
				name: 'Liter'
			},
			min_stock: '0.100',
			shelf_life_days: 1095,
			created_at: Date.now() - 86400000,
			updated_at: Date.now() - 3600000
		},
		total_quantity: 5.5,
		total_value: 825000,
		current_cost: 150000,
		avg_cost: 145000,
		created_at: Date.now() - 86400000, // 1 day ago
		updated_at: Date.now() - 3600000 // 1 hour ago
	},
	{
		uuid: '769ad004-a690-45bd-830f-3485c0db26c5',
		ingredient_uuid: '223e4567-e89b-12d3-a456-426614174001',
		ingredient: {
			uuid: '223e4567-e89b-12d3-a456-426614174001',
			name: 'Unsalted Butter',
			base_uom: {
				uuid: '907e640d-e78e-4f1f-a603-4b2f67b0f268',
				code: 'kg',
				name: 'Kilogram'
			},
			min_stock: '1.000',
			shelf_life_days: 90,
			created_at: Date.now() - 172800000,
			updated_at: Date.now() - 7200000
		},
		total_quantity: 10.0,
		total_value: 1500000,
		current_cost: 150000,
		avg_cost: 145000,
		created_at: Date.now() - 172800000, // 2 days ago
		updated_at: Date.now() - 7200000 // 2 hours ago
	},
	{
		uuid: 'c9b5a6d8-e3f2-4c1a-b9d8-7e6f5d4c3b2a',
		ingredient_uuid: '323e4567-e89b-12d3-a456-426614174002',
		ingredient: {
			uuid: '323e4567-e89b-12d3-a456-426614174002',
			name: 'All-Purpose Flour',
			base_uom: {
				uuid: '907e640d-e78e-4f1f-a603-4b2f67b0f268',
				code: 'kg',
				name: 'Kilogram'
			},
			min_stock: '5.000',
			shelf_life_days: 365,
			created_at: Date.now() - 259200000,
			updated_at: Date.now() - 10800000
		},
		total_quantity: 25.0,
		total_value: 375000,
		current_cost: 15000,
		avg_cost: 14500,
		created_at: Date.now() - 259200000, // 3 days ago
		updated_at: Date.now() - 10800000 // 3 hours ago
	}
];

export const load: PageServerLoad = async ({ url, cookies }) => {
	const page = parseInt(url.searchParams.get('page') || '1');
	const size = parseInt(url.searchParams.get('size') || '10');
	const search = url.searchParams.get('search') || '';

	let predictions: StoreIngredientPredictionsData | null = null;
	let predictionsMessage = '';

	try {
		// Get auth token from cookies
		const token = cookies.get('access_token');
		if (!token) {
			throw redirect(302, '/login');
		}

		// Call the API to get ingredient stocks
		const cookieStr = `access_token=${token}`;

		try {
			const predictionsResponse = await getStoreIngredientPredictionsApi({
				cookies: cookieStr
			});
			predictions = predictionsResponse.data ?? null;
			predictionsMessage = predictionsResponse.message ?? '';
		} catch (error) {
			if (error instanceof ApiError && error.status === 404) {
				predictions = null;
			} else {
				console.error('Failed to fetch ingredient predictions:', error);
			}
		}

		const response = await getIngredientStocksApi(page, size, search, cookieStr);
		
		return {
			items: response.data,
			total: response.meta.total,
			page: response.meta.current_page,
			pageCount: response.meta.last_page,
			search,
			size,
			predictions,
			predictionsMessage
		};
	} catch (err) {
		// Only log unexpected errors (non-404) to avoid noisy dev logs
		if (!(err instanceof ApiError) || err.status !== 404) {
			console.error('Error loading ingredient stocks:', err);
		}
		
		// Use dummy data for development
		const startIndex = (page - 1) * size;
		const filtered = search
			? dummyIngredientStocks.filter((stock) =>
					stock.ingredient.name.toLowerCase().includes(search.toLowerCase())
			  )
			: dummyIngredientStocks;
		
		return {
			items: filtered.slice(startIndex, startIndex + size),
			total: filtered.length,
			page,
			pageCount: Math.ceil(filtered.length / size),
			search,
			size,
			predictions,
			predictionsMessage
		};
	}
};

export const actions: Actions = {
	predict: async ({ cookies }) => {
		const token = cookies.get('access_token');
		if (!token) {
			throw redirect(302, '/login');
		}

		const cookieStr = `access_token=${token}`;

		try {
			const response = await getStoreIngredientPredictionsApi({
				cookies: cookieStr
			});

			return {
				success: true,
				message: response.message ?? 'Rekomendasi bahan baku berhasil diperbarui.',
				data: response.data ?? null
			};
		} catch (error) {
			if (error instanceof ApiError && error.status === 404) {
				return {
					success: true,
					message: 'Belum ada rekomendasi bahan baku terbaru.',
					data: null
				};
			}

			console.error('Failed to refresh ingredient predictions:', error);

			if (error instanceof ApiError) {
				return fail(error.status || 500, {
					success: false,
					message:
						(error.response?.errors?.detail as string | undefined) ??
						error.message ??
						'Gagal memuat rekomendasi bahan baku.'
				});
			}

			return fail(500, {
				success: false,
				message:
					error instanceof Error ? error.message : 'Gagal memuat rekomendasi bahan baku.'
			});
		}
	},

	delete: async ({ request, cookies }) => {
		const formData = await request.formData();
		const uuid = formData.get('uuid')?.toString();

		if (!uuid) {
			return fail(400, { error: 'UUID is required' });
		}

		try {
			// Get auth token from cookies
			const token = cookies.get('access_token');
			if (!token) {
				throw redirect(302, '/login');
			}

			// Call the API to delete the ingredient stock
			const cookieStr = `access_token=${token}`;
			await deleteIngredientStockApi(uuid, cookieStr);
			
			// For development, update the dummy data
			dummyIngredientStocks = dummyIngredientStocks.filter((stock) => stock.uuid !== uuid);
			
			return { success: true };
		} catch (err) {
			console.error('Error deleting ingredient stock:', err);
			return fail(500, { error: 'Failed to delete ingredient stock' });
		}
	}
};
