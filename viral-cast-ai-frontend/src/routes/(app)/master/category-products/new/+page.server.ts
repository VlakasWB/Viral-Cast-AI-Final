// [ID] Action create produk.
// [EN] Create product action.
import { redirect, fail, type Actions } from '@sveltejs/kit';
import { createProduct, parseProductForm } from '$lib/server/db/products';

export const actions: Actions = {
	default: async ({ request }) => {
		const form = await request.formData();
		const { values, errors } = parseProductForm(form);
		if (Object.keys(errors).length) return fail(400, { errors });

        await createProduct(values);
        // Redirect ke daftar Category Products
        throw redirect(303, '/master/category-products');
    }
};
