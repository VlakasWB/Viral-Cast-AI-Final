import { error, redirect, fail, type Actions } from '@sveltejs/kit';
import { getRecipeItemApi, updateRecipeItemApi, getRecipeSetApi } from '$lib/services/recipe.js';
import { getIngredientsApi } from '$lib/services/ingredient.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies }) => {
    const recipeUuid = params.uuid;
    const itemUuid = params.item_uuid;

	if (!recipeUuid) throw error(400, 'Missing recipe UUID');
	if (!itemUuid) throw error(400, 'Missing recipe item UUID');

    try {
        // Build cookie string containing access token for API services
        const accessToken = cookies.get('access_token');
        const cookieStr = accessToken ? `access_token=${accessToken}` : '';

        // Get recipe item details
        const itemResponse = await getRecipeItemApi(itemUuid, cookieStr);
        if (!itemResponse.data?.recipe_item) throw error(404, 'Recipe item not found');

        // Get recipe set details
        const recipeResponse = await getRecipeSetApi(recipeUuid, cookieStr);
        if (!recipeResponse.data?.recipe_set) throw error(404, 'Recipe set not found');

        // Get ingredients for dropdown (paginated)
        const ingredientsResponse = await getIngredientsApi({ page: 1, limit: 100 }, cookieStr);

		return {
			recipeItem: itemResponse.data.recipe_item,
			recipeSet: recipeResponse.data.recipe_set,
			ingredients: ingredientsResponse.data?.ingredients || []
		};
	} catch (err) {
		console.error('Error loading recipe item:', err);
		throw error(404, 'Recipe item not found');
	}
};

export const actions: Actions = {
    default: async ({ params, request, cookies }) => {
        const recipeUuid = params.uuid;
        const itemUuid = params.item_uuid;

		if (!recipeUuid) throw error(400, 'Missing recipe UUID');
		if (!itemUuid) throw error(400, 'Missing recipe item UUID');

		const form = await request.formData();
		const qty = Number(form.get('qty') ?? 0);
		const waste_pct = Number(form.get('waste_pct') ?? 0) / 100; // Convert percentage to decimal

		// Validation
		if (qty <= 0) {
			return fail(400, {
				errors: { qty: 'Quantity harus lebih besar dari 0' },
				values: { qty, waste_pct: waste_pct * 100 }
			});
		}

		if (waste_pct < 0 || waste_pct > 1) {
			return fail(400, {
				errors: { waste_pct: 'Waste percentage harus antara 0-100%' },
				values: { qty, waste_pct: waste_pct * 100 }
			});
		}

        try {
            // Try to update via API
            const requestData = {
                qty,
                waste_pct
            };

            const accessToken = cookies.get('access_token');
            const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
            await updateRecipeItemApi(itemUuid, requestData, cookieStr);
        } catch (error) {
            console.error('API update failed:', error);
            return fail(500, {
                errors: { general: 'Gagal memperbarui bahan resep. Silakan coba lagi.' },
                values: { qty, waste_pct: waste_pct * 100 }
            });
        }

		// Always redirect to recipe detail page
		throw redirect(303, `/master/recipes/${recipeUuid}`);
	}
};
