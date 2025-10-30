// [ID] List produk + hapus via actions. Query: ?page=&q=
// [EN] Product listing + delete via actions. Query: ?page=&q=
import { redirect, fail, type Actions, type ServerLoad } from '@sveltejs/kit';
import { deleteProduct, listProducts } from '$lib/server/db/products';

export const load: ServerLoad = async ({ url }) => {
	const page = Number(url.searchParams.get('page') ?? '1') || 1;
	const q = String(url.searchParams.get('q') ?? '');
	const size = 10;
	const data = await listProducts({ page, size, q });
	return { ...data, q, size };
};

export const actions: Actions = {
	delete: async ({ request }) => {
		const form = await request.formData();
		const id = String(form.get('id') ?? '');
		if (!id) return fail(400, { message: 'Missing id' });
        await deleteProduct(id);
        // Kembali ke daftar Category Products setelah delete
        throw redirect(303, '/master/category-products');
    }
};
