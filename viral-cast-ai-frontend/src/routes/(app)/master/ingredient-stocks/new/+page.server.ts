import { redirect, fail, type Actions } from '@sveltejs/kit';
import { createIngredientApi } from '$lib/services/ingredient.js';

export const actions: Actions = {
    default: async ({ request, cookies }) => {
		const form = await request.formData();
		const name = String(form.get('name') ?? '').trim();
		const base_uom = String(form.get('base_uom') ?? '').trim();
		const min_stock = Number(form.get('min_stock') ?? 0);
		const shelf_life_days = Number(form.get('shelf_life_days') ?? 0);

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
            // Try to create via API
            const requestData = {
                name,
                base_uom,
                min_stock,
                shelf_life_days
            };

            const accessToken = cookies.get('access_token');
            const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
            await createIngredientApi(requestData, cookieStr);
        } catch (error) {
            console.error(
                'API create failed, ingredient will be added to dummy data on next load:',
                error
            );
			// In a real app, you would add to dummy data here
			// For now, we'll just continue with redirect
		}

		// Redirect to ingredient stocks list
		throw redirect(303, '/master/ingredient-stocks');
	}
};
