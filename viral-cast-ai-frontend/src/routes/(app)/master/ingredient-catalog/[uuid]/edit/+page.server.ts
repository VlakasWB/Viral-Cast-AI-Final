import { error, redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import {
	getIngredientCatalogApi,
	updateIngredientCatalogApi
} from '$lib/services/ingredient-catalog.js';
import { getUOMsApi } from '$lib/services/uom';

const DEFAULT_REDIRECT = '/master/ingredient-catalog';

const resolveRedirectTarget = (target: FormDataEntryValue | string | null | undefined) => {
	if (!target) return DEFAULT_REDIRECT;
	const value = String(target).trim();
	if (!value || !value.startsWith('/') || value.startsWith('//')) {
		return DEFAULT_REDIRECT;
	}
	return value;
};

export const load: PageServerLoad = async ({ params, cookies, fetch, url }) => {
	const uuid = params.uuid;
	if (!uuid) throw error(400, 'Missing ingredient UUID');

	const redirectTo = resolveRedirectTarget(url.searchParams.get('redirectTo'));

	try {
		const accessToken = cookies.get('access_token');
		if (!accessToken) {
			throw redirect(302, '/login');
		}
		const cookieStr = `access_token=${accessToken}`;
		const [ingredientCatalog, uomResponse] = await Promise.all([
			getIngredientCatalogApi(uuid, cookieStr, fetch),
			getUOMsApi(cookieStr, fetch)
		]);
		if (!ingredientCatalog) throw error(404, 'Ingredient not found');
		return { ingredient: ingredientCatalog, uoms: uomResponse?.data ?? [], redirectTo };
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
		const description = String(form.get('description') ?? '').trim();
		const base_uom = String(form.get('base_uom') ?? '').trim();
		const minStockRaw = form.get('min_stock')?.toString().trim() ?? '';
		const shelfLifeRaw = form.get('shelf_life_days')?.toString().trim() ?? '';
		const redirectTo = resolveRedirectTarget(form.get('redirectTo'));

		const min_stock = Number(minStockRaw);
		const shelf_life_days = Number(shelfLifeRaw);

		if (!uuid) throw error(400, 'Missing ingredient UUID');

		// Validation
		if (!name) {
			return fail(400, {
				errors: { name: 'Ingredient name is required' },
				values: { name, description, base_uom, min_stock: minStockRaw, shelf_life_days: shelfLifeRaw }
			});
		}

		if (!base_uom) {
			return fail(400, {
				errors: { base_uom: 'Base UOM is required' },
				values: { name, description, base_uom, min_stock: minStockRaw, shelf_life_days: shelfLifeRaw }
			});
		}

		if (!minStockRaw || Number.isNaN(min_stock) || min_stock <= 0) {
			return fail(400, {
				errors: { min_stock: 'Minimum stock must be greater than 0' },
				values: { name, description, base_uom, min_stock: minStockRaw, shelf_life_days: shelfLifeRaw }
			});
		}

		if (!shelfLifeRaw || Number.isNaN(shelf_life_days) || shelf_life_days <= 0) {
			return fail(400, {
				errors: { shelf_life_days: 'Shelf life must be greater than 0 days' },
				values: { name, description, base_uom, min_stock: minStockRaw, shelf_life_days: shelfLifeRaw }
			});
		}

		try {
			const updateData = {
				name,
				description: description || undefined,
				base_uom_uuid: base_uom,
				minimum_stock: minStockRaw,
				shelf_life_days
			};

			const accessToken = cookies.get('access_token');
			const cookieStr = accessToken ? `access_token=${accessToken}` : null;
			if (cookieStr) {
				await updateIngredientCatalogApi(uuid, updateData, cookieStr);
			} else {
				await updateIngredientCatalogApi(uuid, updateData);
			}
		} catch (error) {
			console.error('Error updating ingredient:', error);
			return fail(500, {
				errors: { general: 'Failed to update ingredient' },
				values: { name, description, base_uom, min_stock: minStockRaw, shelf_life_days: shelfLifeRaw }
			});
		}

		// Redirect back to ingredient catalog list
		throw redirect(303, redirectTo);
	}
};
