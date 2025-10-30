import { error, redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getCategoryApi, updateCategoryApi } from '$lib/services/category.js';

export const load: PageServerLoad = async ({ params, cookies }) => {
	const uuid = params.uuid;
	if (!uuid) throw error(400, 'Missing category UUID');

	try {
		const token = cookies.get('access_token');
		const cookieStr = token ? `access_token=${token}` : undefined;
		const response = await getCategoryApi(uuid, cookieStr);
		if (!response.data) throw error(404, 'Category not found');
		return { category: response.data };
	} catch (err) {
		console.error('Error loading category:', err);
		throw error(404, 'Category not found');
	}
};

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const form = await request.formData();
		const uuid = String(form.get('uuid') ?? '');
		const name = String(form.get('name') ?? '').trim();

		if (!uuid) throw error(400, 'Missing category UUID');

		// Validation
		if (!name) {
			return fail(400, {
				errors: { name: 'Category name is required' },
				values: { name }
			});
		}

		try {
			const token = cookies.get('access_token');
			const cookieStr = token ? `access_token=${token}` : undefined;
			await updateCategoryApi(uuid, { name }, cookieStr);
		} catch (error) {
			console.error('Error updating category:', error);
			return fail(500, {
				errors: { general: 'Failed to update category' },
				values: { name }
			});
		}

		throw redirect(303, '/master/categories');
	}
};
