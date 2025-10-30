import { error, redirect, type Actions, type ServerLoad } from '@sveltejs/kit';
import { getProduct, parseProductForm, updateProduct } from '$lib/server/db/products';

export const load: ServerLoad = async ({ params }) => {
	const id = params.id; // [CHANGED]
	if (!id) throw error(400, 'Missing id param'); // [CHANGED]

	const product = await getProduct(id); // [CHANGED]
	if (!product) throw error(404, 'Product not found');
	return { product };
};

export const actions: Actions = {
	default: async ({ request }) => {
		const data = await request.formData();
		const id = String(data.get('id') ?? '');
		if (!id) throw error(400, 'Missing id');

		const { values, errors } = parseProductForm(data);
		if (Object.keys(errors).length) return { errors, id };

        await updateProduct(id, values);
        // Redirect ke daftar Category Products
        throw redirect(303, '/master/category-products');
    }
};
