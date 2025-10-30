import { redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { createIngredientStockMoveApi } from '$lib/services/ingredient-stock-moves';
import { getIngredientCatalogsApi } from '$lib/services/ingredient-catalog';

// Load data untuk form pembuatan ingredient stock move
export const load: PageServerLoad = async ({ locals, cookies, fetch }) => {
    // Pastikan user sudah login
    const session = await locals.getSession();
    if (!session) {
        throw redirect(302, '/login');
    }

    try {
        // Ambil ingredient catalogs untuk dropdown
		const accessToken = cookies.get('access_token');
		const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
		const ingredients = await getIngredientCatalogsApi(1, 100, '', cookieStr, fetch);

        return {
            ingredients: ingredients.items || []
        };
    } catch (error) {
        console.error('Error loading data:', error);
        return {
            stores: [],
            ingredients: []
        };
    }
};

// Actions untuk form pembuatan ingredient stock move
export const actions: Actions = {
	default: async ({ request, locals, cookies }) => {
        // Pastikan user sudah login
        const session = await locals.getSession();
        if (!session) {
            throw redirect(302, '/login');
        }

        const formData = await request.formData();
        const ingredientUuid = formData.get('ingredientUuid')?.toString() || '';
        const moveType = formData.get('moveType')?.toString() as 'PRODUCTION' | 'ADJUSTMENT' | 'WASTE' | 'RETURN';
		const quantity = formData.get('quantity')?.toString() || '';
		const price = formData.get('price')?.toString() || '';

        // Validasi input
        const errors: Record<string, string> = {};
        
        if (!ingredientUuid) {
            errors.ingredientUuid = 'Ingredient harus dipilih';
        }

        if (!moveType) {
            errors.moveType = 'Tipe pergerakan harus dipilih';
        }

		if (!quantity || isNaN(parseFloat(quantity)) || parseFloat(quantity) <= 0) {
			errors.quantity = 'Quantity harus berupa angka positif';
		}

		if (!price || isNaN(parseFloat(price)) || parseFloat(price) <= 0) {
			errors.price = 'Harga harus berupa angka positif';
		}

		if (Object.keys(errors).length > 0) {
			return fail(400, { errors, ingredientUuid, moveType, quantity, price });
        }

        try {
			const accessToken = cookies.get('access_token');
			if (!accessToken) {
				throw redirect(302, '/login');
			}
			const cookieStr = `access_token=${accessToken}`;
            // Kirim data ke API
			await createIngredientStockMoveApi(
				{
					ingredient_uuid: ingredientUuid,
					ingredient_catalog_uuid: ingredientUuid,
					move_type: moveType as 'PRODUCTION' | 'ADJUSTMENT' | 'WASTE' | 'RETURN',
					ref_type: moveType,
					quantity: parseFloat(quantity),
					price: parseFloat(price),
					price_updated_at: Date.now(),
					effective_at: Date.now()
				},
				cookieStr
			);

            return { success: true };
        } catch (error) {
            console.error('Error creating ingredient stock move:', error);
            return fail(500, { 
                error: 'Gagal membuat pergerakan stok ingredient', 
                ingredientUuid, 
                moveType, 
                quantity,
				price
            });
        }
    }
};
