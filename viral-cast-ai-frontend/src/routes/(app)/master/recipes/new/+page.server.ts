import { redirect, fail, type Actions } from '@sveltejs/kit';
import { createRecipeSetApi } from '$lib/services/recipe.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
    // No preloading needed since recipe creation no longer requires a product.
    return {};
};

export const actions: Actions = {
    default: async ({ request, cookies }) => {
    const form = await request.formData();
    const name = String(form.get('name') ?? '').trim();
    const yield_qty = Number(form.get('yield_qty') ?? 0);
    const effective_from = String(form.get('effective_from') ?? '').trim();
    const effective_to = String(form.get('effective_to') ?? '').trim();
    const is_active = form.get('is_active') === 'on';

		// Check if effective dates are provided (optional)
		const hasEffectiveDates = effective_from && effective_to;

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
            await createRecipeSetApi(requestData, cookieStr);
        } catch (error) {
            console.error('API create failed:', error);
            return fail(500, {
                errors: { general: 'Failed to create recipe. Please try again.' },
                values: { name, yield_qty, effective_from, effective_to, is_active }
            });
        }

		// Always redirect to recipes page
		throw redirect(303, '/master/recipes');
	}
};
