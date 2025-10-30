import { redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import {
	getIngredientCatalogsApi,
	deleteIngredientCatalogApi
} from '$lib/services/ingredient-catalog.js';

// Fungsi untuk memuat data ingredient catalog
export const load: PageServerLoad = async ({ url, locals, cookies, fetch }) => {
    // Pastikan user sudah login
    const session = await locals.getSession();
    if (!session) {
        throw redirect(302, '/login');
    }

	// Ambil parameter query
	const page = Number(url.searchParams.get('page') || '1');
	const search = url.searchParams.get('search') || '';
	const sizeQuery = url.searchParams.get('size');
	const parsedSize = sizeQuery ? Number(sizeQuery) : Number.NaN;
	const DEFAULT_PAGE_SIZE = 10;
	const normalizedSize =
		Number.isFinite(parsedSize) && parsedSize > 0 ? parsedSize : DEFAULT_PAGE_SIZE;

    try {
        // Ambil data dari API
		const accessToken = cookies.get('access_token');
		const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
		const data = await getIngredientCatalogsApi(page, normalizedSize, search, cookieStr, fetch);
        return {
            items: data.items,
            total: data.total,
            page: data.page,
            pageCount: data.pageCount,
            search,
			size: normalizedSize
        };
    } catch (error) {
        console.error('Error loading ingredient catalogs:', error);
        return {
            items: [],
            total: 0,
            page: 1,
            pageCount: 1,
            search,
			size: normalizedSize
        };
    }
};

// Definisi actions untuk operasi CRUD
export const actions: Actions = {
    // Action untuk menghapus ingredient catalog
	delete: async ({ request, locals, cookies }) => {
        // Pastikan user sudah login
        const session = await locals.getSession();
        if (!session) {
            throw redirect(302, '/login');
        }

        const data = await request.formData();
        const uuid = data.get('uuid')?.toString();

        if (!uuid) {
            return fail(400, { error: 'UUID tidak valid' });
        }

        try {
			const accessToken = cookies.get('access_token');
			const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
			await deleteIngredientCatalogApi(uuid, cookieStr);
            return { success: true };
        } catch (error) {
            console.error('Error deleting ingredient catalog:', error);
            return fail(500, { error: 'Gagal menghapus katalog bahan' });
        }
    }
};
