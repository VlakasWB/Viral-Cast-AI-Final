import { redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getProductsApi, deleteProductApi } from '$lib/services/product.js';
import { getStoreProductPredictionsApi } from '$lib/services/storeProductPredictions';
import type { Product } from '$lib/types/product.js';
import type { StoreProductPredictionsData } from '$lib/types/store-product-prediction';
import { ApiError } from '$lib/utils/api';

// Global dummy data store (in real app, this would be a database)
let dummyProducts: Product[] = [
	{
		uuid: '9d9246b5-85cf-42cf-a0f9-a7018eab1cd9',
		name: 'Premium Coffee Blend',
		category_uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		sku: 'PCB-002',
		price: '85000.00',
		current_recipe_uuid: 'recipe-premium-coffee',
		status: 'ACTIVE',
		image_url: 'https://example.com/images/coffee-blend-1.jpg',
		created_at: Date.now() - 86400000, // 1 day ago
		updated_at: Date.now() - 3600000 // 1 hour ago
	},
	{
		uuid: 'd780f3f7-d5b7-457a-ac6c-8450a6a1be3d',
		name: 'Artisan Espresso Beans',
		category_uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		sku: 'AEB-001',
		price: '120000.00',
		current_recipe_uuid: null,
		status: 'ACTIVE',
		image_url: null,
		created_at: Date.now() - 172800000, // 2 days ago
		updated_at: Date.now() - 7200000 // 2 hours ago
	},
	{
		uuid: 'f1a2b3c4-d5e6-7890-abcd-ef1234567890',
		name: 'Organic Green Tea',
		category_uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		sku: 'OGT-003',
		price: '45000.00',
		current_recipe_uuid: 'recipe-organic-green-tea',
		status: 'ACTIVE',
		image_url: 'https://example.com/images/green-tea.jpg',
		created_at: Date.now() - 259200000, // 3 days ago
		updated_at: Date.now() - 10800000 // 3 hours ago
	},
	{
		uuid: 'a1b2c3d4-e5f6-7890-1234-567890abcdef',
		name: 'Chocolate Croissant',
		category_uuid: '2a8ceed7-1b26-48ac-a191-04d31f254419',
		sku: 'CC-004',
		price: '25000.00',
		current_recipe_uuid: 'recipe-chocolate-croissant',
		status: 'INACTIVE',
		image_url: null,
		created_at: Date.now() - 345600000, // 4 days ago
		updated_at: Date.now() - 14400000 // 4 hours ago
	},
	{
		uuid: 'b2c3d4e5-f6a7-8901-2345-6789abcdef01',
		name: 'Fresh Bagel',
		category_uuid: '2a8ceed7-1b26-48ac-a191-04d31f254419',
		sku: 'FB-005',
		price: '15000.00',
		current_recipe_uuid: null,
		status: 'ACTIVE',
		image_url: 'https://example.com/images/bagel.jpg',
		created_at: Date.now() - 432000000, // 5 days ago
		updated_at: Date.now() - 18000000 // 5 hours ago
	}
];

export const load: PageServerLoad = async ({ url, cookies }) => {
	const page = Number(url.searchParams.get('page') ?? '1') || 1;
	const search = String(url.searchParams.get('search') ?? '');
	const sizeParam = Number(url.searchParams.get('size') ?? '10');
	const size = Number.isFinite(sizeParam) && sizeParam > 0 ? sizeParam : 10;

	const token = cookies.get('access_token');
	const cookieStr = token ? `access_token=${token}` : undefined;

	let predictions: StoreProductPredictionsData | null = null;
	let predictionsMessage = '';

	try {
		const predictionsResponse = await getStoreProductPredictionsApi({
			cookies: cookieStr
		});
		predictions = predictionsResponse.data ?? null;
		predictionsMessage = predictionsResponse.message ?? '';
	} catch (error) {
		if (error instanceof ApiError && error.status === 404) {
			predictions = null;
		} else {
			console.error('Failed to fetch store product predictions:', error);
		}
	}

	try {
		const response = await getProductsApi(
			{
				page,
				limit: size,
				sort_by: 'name',
				sort_order: 'asc',
				is_active: true,
				search
			},
			cookieStr
		);

		const allItems = response.data.products || [];
		const total = response.data.pagination?.total ?? allItems.length;
		const pageCount = response.data.pagination?.total_pages ?? Math.ceil(total / size);

		return {
			items: allItems,
			total,
			page,
			pageCount,
			search,
			size,
			predictions,
			predictionsMessage
		};
	} catch (error) {
		console.error('API not available, using dummy data:', error);

		const total = dummyProducts.length;
		const pageCount = Math.ceil(total / size);
		const start = (page - 1) * size;
		const end = start + size;
		const items = dummyProducts.slice(start, end);

		return {
			items,
			total,
			page,
			pageCount,
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
		const cookieStr = token ? `access_token=${token}` : undefined;

		try {
			const response = await getStoreProductPredictionsApi({
				cookies: cookieStr
			});

			return {
				success: true,
				message: response.message ?? 'Prediksi produk berhasil diperbarui.',
				data: response.data ?? null
			};
		} catch (error) {
			if (error instanceof ApiError && error.status === 404) {
				return {
					success: true,
					message: 'Belum ada prediksi produk terbaru.',
					data: null
				};
			}

			console.error('Failed to refresh store product predictions:', error);

			if (error instanceof ApiError) {
				return fail(error.status || 500, {
					success: false,
					message:
						(error.response?.errors?.detail as string | undefined) ??
						error.message ??
						'Gagal memuat prediksi produk.'
				});
			}

			return fail(500, {
				success: false,
				message: error instanceof Error ? error.message : 'Gagal memuat prediksi produk.'
			});
		}
	},

	delete: async ({ request, cookies }) => {
		const form = await request.formData();
		const uuid = String(form.get('uuid') ?? '');

		if (!uuid) {
			return fail(400, { message: 'Missing product UUID' });
		}

		try {
			// Try to delete via API
			const token = cookies.get('access_token');
			const cookieStr = token ? `access_token=${token}` : undefined;
			await deleteProductApi(uuid, cookieStr);
		} catch (error) {
			console.error('API delete failed, removing from dummy data:', error);
			// Remove from dummy data as fallback
			dummyProducts = dummyProducts.filter((product) => product.uuid !== uuid);
		}

		// Always redirect back to products page
		throw redirect(303, '/master/products');
	}
};
