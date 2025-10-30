import { error, redirect, fail, type Actions } from '@sveltejs/kit';
import { getRecipeSetApi, updateRecipeSetApi } from '$lib/services/recipe.js';
import { getProductsApi } from '$lib/services/product.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies }) => {
	const uuid = params.uuid;
	if (!uuid) throw error(400, 'Missing recipe UUID');

	try {
		const accessToken = cookies.get('access_token');
		const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
		// Get recipe set details
		const recipeResponse = await getRecipeSetApi(uuid, cookieStr);
		if (!recipeResponse.data?.recipe_set) throw error(404, 'Recipe set not found');

		// Get products for dropdown
		const productsResponse = await getProductsApi(
			{ page: 1, limit: 100, is_active: true },
			cookieStr
		);

		return {
			recipeSet: recipeResponse.data.recipe_set,
			products: productsResponse.data.products || []
		};
	} catch (err) {
		console.error('Error loading recipe set:', err);
		throw error(404, 'Recipe set not found');
	}
};

export const actions: Actions = {
    default: async ({ params, request, cookies }) => {
        const uuid = params.uuid;
        if (!uuid) throw error(400, 'Missing recipe UUID');

		const form = await request.formData();
		const name = String(form.get('name') ?? '').trim();
		const yield_qty = Number(form.get('yield_qty') ?? 0);
		const effective_from = String(form.get('effective_from') ?? '').trim();
		const effective_to = String(form.get('effective_to') ?? '').trim();
		const is_active = form.get('is_active') === 'on';

		// Check if effective dates are provided (optional)
		const hasEffectiveDates = effective_from && effective_to;

		// Validation
		if (!name) {
			return fail(400, {
				errors: { name: 'Recipe name is required' },
				values: { name, yield_qty, effective_from, effective_to, is_active }
			});
		}

		if (yield_qty <= 0) {
			return fail(400, {
				errors: { yield_qty: 'Yield quantity must be greater than 0' },
				values: { name, yield_qty, effective_from, effective_to, is_active }
			});
		}

		let requestData: any = {
			name,
			yield_qty,
			is_active
		};

		// Only validate and add effective dates if they are provided
		if (hasEffectiveDates) {
			// Convert dates to timestamps
			const effectiveFromTimestamp = new Date(effective_from).getTime();
			const effectiveToTimestamp = new Date(effective_to).getTime();

			if (effectiveToTimestamp <= effectiveFromTimestamp) {
				return fail(400, {
					errors: { effective_to: 'End date must be greater than start date' },
					values: { name, yield_qty, effective_from, effective_to, is_active }
				});
			}

			requestData.effective_from = effectiveFromTimestamp;
			requestData.effective_to = effectiveToTimestamp;
		}

        try {
            const accessToken = cookies.get('access_token');
            const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
            await updateRecipeSetApi(uuid, requestData, cookieStr);
        } catch (error) {
            console.error('API update failed:', error);
            return fail(500, {
                errors: { general: 'Failed to update recipe. Please try again.' },
                values: { name, yield_qty, effective_from, effective_to, is_active }
            });
        }

		// Always redirect to recipe detail page
		throw redirect(303, `/master/recipes/${uuid}`);
	}
};
