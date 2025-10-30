import { error, redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import {
	getIngredientStockMoveApi,
	updateIngredientStockMoveApi
} from '$lib/services/ingredient-stock-moves';
import { getIngredientCatalogsApi } from '$lib/services/ingredient-catalog';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const uuid = params.uuid;
	if (!uuid) throw error(400, 'Missing stock move UUID');

	try {
		const accessToken = cookies.get('access_token');
		if (!accessToken) {
			throw redirect(302, '/auth/login');
		}

		const cookieStr = `access_token=${accessToken}`;
		
		// Fetch stock move and ingredients in parallel
		const [stockMove, ingredientsResponse] = await Promise.all([
			getIngredientStockMoveApi(uuid, cookieStr, fetch),
			getIngredientCatalogsApi(1, 100, '', cookieStr, fetch)
		]);

		if (!stockMove) throw error(404, 'Stock move not found');

		return {
			stockMove,
			ingredients: ingredientsResponse.items || []
		};
	} catch (err) {
		console.error('Error loading stock move:', err);
		throw error(404, 'Stock move not found');
	}
};

export const actions: Actions = {
	default: async ({ request, cookies, params }) => {
		const form = await request.formData();
		const uuid = params.uuid;
		const ingredientUuid = String(form.get('ingredientUuid') ?? '').trim();
		const moveType = String(form.get('moveType') ?? '').trim();
		const quantity = Number(form.get('quantity') ?? 0);
		const price = Number(form.get('price') ?? 0);

		if (!uuid) throw error(400, 'Missing stock move UUID');

		// Validation
		if (!ingredientUuid) {
			return fail(400, {
				errors: { ingredientUuid: 'Ingredient is required' },
				values: { ingredientUuid, moveType, quantity }
			});
		}

		if (!moveType || !['PRODUCTION', 'ADJUSTMENT', 'WASTE', 'RETURN'].includes(moveType)) {
			return fail(400, {
				errors: { moveType: 'Valid move type is required (PRODUCTION/ADJUSTMENT/WASTE/RETURN)' },
				values: { ingredientUuid, moveType, quantity }
			});
		}

		if (!quantity || quantity <= 0) {
			return fail(400, {
				errors: { quantity: 'Quantity must be greater than 0' },
				values: { ingredientUuid, moveType, quantity, price }
			});
		}

		if (!price || price <= 0) {
			return fail(400, {
				errors: { price: 'Price must be greater than 0' },
				values: { ingredientUuid, moveType, quantity, price }
			});
		}

		try {
			const accessToken = cookies.get('access_token');
			if (!accessToken) {
				throw redirect(302, '/auth/login');
			}

			const cookieStr = `access_token=${accessToken}`;
			const updateData = {
				ingredient_uuid: ingredientUuid,
				move_type: moveType as 'PRODUCTION' | 'ADJUSTMENT' | 'WASTE' | 'RETURN',
				ref_type: moveType,
				quantity,
				price,
				price_updated_at: Date.now(),
				effective_at: Date.now()
			};

			await updateIngredientStockMoveApi(uuid, updateData, cookieStr);
		} catch (err) {
			console.error('Error updating stock move:', err);
			return fail(500, {
				errors: { general: 'Failed to update stock move' },
				values: { ingredientUuid, moveType, quantity, price }
			});
		}

        // Redirect ke halaman daftar setelah berhasil update
        throw redirect(303, `/master/ingredient-stock-moves`);
    }
};
