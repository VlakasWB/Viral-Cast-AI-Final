import { redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { createIngredientCatalogApi } from '$lib/services/ingredient-catalog.js';
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

export const load: PageServerLoad = async ({ locals, cookies, fetch, url }) => {
	const session = await locals.getSession?.();
	if (!session) {
		throw redirect(302, '/login');
	}

	const redirectTo = resolveRedirectTarget(url.searchParams.get('redirectTo'));

	try {
		const accessToken = cookies.get('access_token');
		if (!accessToken) {
			throw redirect(302, '/login');
		}
		const cookieStr = `access_token=${accessToken}`;
		const uomResponse = await getUOMsApi(cookieStr, fetch);
		return {
			uoms: uomResponse?.data ?? [],
			redirectTo
		};
	} catch (error) {
		console.error('Error loading UOMs for ingredient form:', error);
		return { uoms: [], redirectTo };
	}
};

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const form = await request.formData();
		const name = String(form.get('name') ?? '').trim();
		const description = String(form.get('description') ?? '').trim();
		const base_uom = String(form.get('base_uom') ?? '').trim();
		const minStockRaw = form.get('min_stock')?.toString().trim() ?? '';
		const shelfLifeRaw = form.get('shelf_life_days')?.toString().trim() ?? '';
		const redirectTo = resolveRedirectTarget(form.get('redirectTo'));

		const min_stock = Number(minStockRaw);
		const shelf_life_days = Number(shelfLifeRaw);

		const invalidMinStock = !minStockRaw || Number.isNaN(min_stock) || min_stock <= 0;
		const invalidShelfLife =
			!shelfLifeRaw || Number.isNaN(shelf_life_days) || shelf_life_days <= 0;

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

		if (invalidMinStock) {
			return fail(400, {
				errors: { min_stock: 'Minimum stock must be greater than 0' },
				values: { name, description, base_uom, min_stock: minStockRaw, shelf_life_days: shelfLifeRaw }
			});
		}

		if (invalidShelfLife) {
			return fail(400, {
				errors: { shelf_life_days: 'Shelf life must be greater than 0 days' },
				values: { name, description, base_uom, min_stock: minStockRaw, shelf_life_days: shelfLifeRaw }
			});
		}

		try {
			const requestData = {
				name,
				description: description || undefined,
				base_uom_uuid: base_uom,
				minimum_stock: minStockRaw,
				shelf_life_days
			};

			const accessToken = cookies.get('access_token');
			const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
			await createIngredientCatalogApi(requestData, cookieStr);
		} catch (error) {
			console.error(
				'API create failed, ingredient will be added to dummy data on next load:',
				error
			);
			return fail(500, {
				errors: { api: 'Failed to create ingredient catalog' },
				values: { name, description, base_uom, min_stock: minStockRaw, shelf_life_days: shelfLifeRaw }
			});
		}

		// Redirect to ingredient catalog list
		throw redirect(303, redirectTo);
	}
};
