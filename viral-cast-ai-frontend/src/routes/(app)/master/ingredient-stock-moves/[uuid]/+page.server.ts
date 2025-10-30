import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getIngredientStockMoveApi } from '$lib/services/ingredient-stock-moves';

// Load data untuk halaman detail ingredient stock move
export const load: PageServerLoad = async ({ params, locals, cookies, fetch }) => {
    // Pastikan user sudah login
    const session = await locals.getSession();
    if (!session) {
        throw redirect(302, '/login');
    }

    const { uuid } = params;

    try {
		const accessToken = cookies.get('access_token');
		if (!accessToken) {
			throw redirect(302, '/login');
		}
		const cookieStr = `access_token=${accessToken}`;
        // Ambil data ingredient stock move berdasarkan UUID
		const stockMove = await getIngredientStockMoveApi(uuid, cookieStr, fetch);
        
        return {
            stockMove
        };
    } catch (error) {
        console.error('Error loading ingredient stock move:', error);
        throw redirect(302, '/master/ingredient-stock-moves');
    }
};
