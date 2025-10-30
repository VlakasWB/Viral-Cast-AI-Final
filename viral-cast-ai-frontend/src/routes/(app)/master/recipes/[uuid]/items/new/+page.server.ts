import { redirect, fail, type Actions } from '@sveltejs/kit';
import { createRecipeItemApi, getRecipeSetApi } from '$lib/services/recipe.js';
import { getIngredientsApi } from '$lib/services/ingredient.js';
import { getUOMsApi } from '$lib/services/uom.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies }) => {
    const recipeUuid = params.uuid;
    if (!recipeUuid) throw new Error('Missing recipe UUID');

    try {
        // Build cookie string containing access token for API services
        const accessToken = cookies.get('access_token');
        const cookieStr = accessToken ? `access_token=${accessToken}` : '';

        // Get recipe set details
        const recipeResponse = await getRecipeSetApi(recipeUuid, cookieStr);
        if (!recipeResponse.data?.recipe_set) throw new Error('Recipe set not found');

        // Get ingredients for dropdown (paginated)
        const ingredientsResponse = await getIngredientsApi({ page: 1, limit: 100 }, cookieStr);

        // Get UOMs for dropdown
        const uomsResponse = await getUOMsApi(cookieStr);

		return {
			recipeSet: recipeResponse.data.recipe_set,
			ingredients: ingredientsResponse.data?.ingredients || [],
			uoms: uomsResponse.data || []
		};
	} catch (error) {
		console.error('Error loading data:', error);
		return {
			recipeSet: null,
			ingredients: [],
			uoms: []
		};
	}
};

export const actions: Actions = {
    default: async ({ params, request, cookies }) => {
        const recipeUuid = params.uuid;
        if (!recipeUuid) throw new Error('Missing recipe UUID');

		const form = await request.formData();
		const ingredient_uuid = String(form.get('ingredient_uuid') ?? '').trim();
		const uom_uuid = String(form.get('uom_uuid') ?? '').trim();
		const qty = Number(form.get('qty') ?? 0);
		const waste_pct = Number(form.get('waste_pct') ?? 0) / 100; // Convert percentage to decimal

		// Validation
		if (!ingredient_uuid) {
			return fail(400, {
				errors: { ingredient_uuid: 'Bahan harus dipilih' },
				values: { ingredient_uuid, uom_uuid, qty, waste_pct: waste_pct * 100 }
			});
		}

		if (!uom_uuid) {
			return fail(400, {
				errors: { uom_uuid: 'Unit of Measure harus dipilih' },
				values: { ingredient_uuid, uom_uuid, qty, waste_pct: waste_pct * 100 }
			});
		}

		if (qty <= 0) {
			return fail(400, {
				errors: { qty: 'Quantity harus lebih besar dari 0' },
				values: { ingredient_uuid, uom_uuid, qty, waste_pct: waste_pct * 100 }
			});
		}

		if (waste_pct < 0 || waste_pct > 1) {
			return fail(400, {
				errors: { waste_pct: 'Waste percentage harus antara 0-100%' },
				values: { ingredient_uuid, uom_uuid, qty, waste_pct: waste_pct * 100 }
			});
		}

        try {
            // Try to create via API
            const requestData = {
                recipe_uuid: recipeUuid,
                ingredient_uuid,
                uom_uuid,
                qty,
                waste_pct
            };

            const accessToken = cookies.get('access_token');
            const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
            await createRecipeItemApi(requestData, cookieStr);
        } catch (error) {
            console.error('API create failed:', error);
            return fail(500, {
                errors: { general: 'Gagal menambahkan bahan ke resep. Silakan coba lagi.' },
                values: { ingredient_uuid, uom_uuid, qty, waste_pct: waste_pct * 100 }
            });
        }

		// Always redirect to recipe detail page
		throw redirect(303, `/master/recipes/${recipeUuid}`);
	}
};
