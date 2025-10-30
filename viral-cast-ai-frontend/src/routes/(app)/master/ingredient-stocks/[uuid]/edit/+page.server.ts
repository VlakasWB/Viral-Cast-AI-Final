import { error, redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getIngredientApi, updateIngredientApi } from '$lib/services/ingredient.js';

export const load: PageServerLoad = async ({ params, cookies }) => {
	const uuid = params.uuid;
	if (!uuid) throw error(400, 'Missing ingredient UUID');

	try {
		const accessToken = cookies.get('access_token');
		const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
		const response = await getIngredientApi(uuid, cookieStr);
		if (!response.data) throw error(404, 'Ingredient not found');
		return { ingredient: response.data };
	} catch (err) {
		console.error('Error loading ingredient:', err);
		throw error(404, 'Ingredient not found');
	}
};

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const form = await request.formData();
		const uuid = String(form.get('uuid') ?? '');
		const name = String(form.get('name') ?? '').trim();
		const base_uom = String(form.get('base_uom') ?? '').trim();
		const min_stock = Number(form.get('min_stock') ?? 0);
		const shelf_life_days = Number(form.get('shelf_life_days') ?? 0);

		if (!uuid) throw error(400, 'Missing ingredient UUID');

		// Validation
		if (!name) {
			return fail(400, {
				errors: { name: 'Ingredient name is required' },
				values: { name, base_uom, min_stock, shelf_life_days }
			});
		}

		if (!base_uom) {
			return fail(400, {
				errors: { base_uom: 'Base UOM is required' },
				values: { name, base_uom, min_stock, shelf_life_days }
			});
		}

		if (min_stock <= 0) {
			return fail(400, {
				errors: { min_stock: 'Minimum stock must be greater than 0' },
				values: { name, base_uom, min_stock, shelf_life_days }
			});
		}

		if (shelf_life_days <= 0) {
			return fail(400, {
				errors: { shelf_life_days: 'Shelf life must be greater than 0 days' },
				values: { name, base_uom, min_stock, shelf_life_days }
			});
		}

		try {
			const updateData = {
				name,
				base_uom,
				min_stock,
				shelf_life_days
			};

			const accessToken = cookies.get('access_token');
			const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
			await updateIngredientApi(uuid, updateData, cookieStr);
		} catch (error) {
			console.error('Error updating ingredient:', error);
			return fail(500, {
				errors: { general: 'Failed to update ingredient' },
				values: { name, base_uom, min_stock, shelf_life_days }
			});
		}

		// Redirect back to ingredient stocks list
		throw redirect(303, '/master/ingredient-stocks');
	}
};
