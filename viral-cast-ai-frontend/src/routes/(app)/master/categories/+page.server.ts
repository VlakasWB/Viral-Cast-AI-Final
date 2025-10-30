import { redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getCategoriesApi, deleteCategoryApi } from '$lib/services/category.js';
import type { Category } from '$lib/types/category.js';

// Global dummy data store (in real app, this would be a database)
let dummyCategories: Category[] = [
	{
		uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		name: 'Beverages',
		created_at: Date.now() - 86400000,
		updated_at: Date.now() - 3600000
	},
	{
		uuid: '2a8ceed7-1b26-48ac-a191-04d31f254419',
		name: 'Bakery',
		created_at: Date.now() - 172800000,
		updated_at: Date.now() - 7200000
	},
	{
		uuid: '300b630f-2e59-48be-b3ad-2844c3286547',
		name: 'Snacks',
		created_at: Date.now() - 259200000,
		updated_at: Date.now() - 10800000
	},
	{
		uuid: '690799f5-9eb6-49d9-9ca3-349a3428e191',
		name: 'Desserts',
		created_at: Date.now() - 345600000,
		updated_at: Date.now() - 14400000
	}
];

export const load: PageServerLoad = async ({ url, cookies }) => {
	const page = Number(url.searchParams.get('page') ?? '1') || 1;
	const q = String(url.searchParams.get('q') ?? '');
	const size = 10;

	try {
		// Try to get categories from API
		const token = cookies.get('access_token');
		const cookieStr = token ? `access_token=${token}` : undefined;
		const response = await getCategoriesApi(cookieStr);

		// Return API data if available
		const allItems = response.data || [];
		const total = allItems.length;
		const pageCount = Math.ceil(total / size);

		return {
			items: allItems,
			total,
			page,
			pageCount,
			q,
			size
		};
	} catch (error) {
		console.error('API not available, using dummy data:', error);

		// Always return dummy data when API is not available
		const total = dummyCategories.length;
		const pageCount = Math.ceil(total / size);

		return {
			items: dummyCategories,
			total,
			page,
			pageCount,
			q,
			size
		};
	}
};

export const actions: Actions = {
	delete: async ({ request, cookies }) => {
		const form = await request.formData();
		const uuid = String(form.get('uuid') ?? '');

		if (!uuid) {
			return fail(400, { message: 'Missing category UUID' });
		}

		try {
			// Try to delete via API
			const token = cookies.get('access_token');
			const cookieStr = token ? `access_token=${token}` : undefined;
			await deleteCategoryApi(uuid, cookieStr);
		} catch (error) {
			console.error('API delete failed, removing from dummy data:', error);
			// Remove from dummy data as fallback
			dummyCategories = dummyCategories.filter((cat) => cat.uuid !== uuid);
		}

		// Always redirect back to categories page
		throw redirect(303, '/master/categories');
	}
};
