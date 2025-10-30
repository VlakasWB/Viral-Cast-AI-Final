import { redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getIngredientCatalogApi, updateIngredientCatalogApi } from '$lib/services/ingredient-catalog.js';
import { getUOMsApi } from '$lib/services/uom.js';

// Load data untuk halaman detail ingredient catalog
export const load: PageServerLoad = async ({ params, locals, cookies, fetch }) => {
    // Pastikan user sudah login
    const session = await locals.getSession();
    if (!session) {
        throw redirect(302, '/login');
    }

    const { uuid } = params;

	try {
		// Ambil data ingredient catalog berdasarkan UUID
		const accessToken = cookies.get('access_token');
		const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
		const ingredientCatalog = await getIngredientCatalogApi(uuid, cookieStr, fetch);

		// Ambil data UOM untuk dropdown
		const uomsResponse = await getUOMsApi(cookieStr, fetch);
        
        return {
            ingredientCatalog,
            uoms: uomsResponse.data ?? []
        };
    } catch (error) {
        console.error('Error loading ingredient catalog:', error);
        throw redirect(302, '/master/ingredient-catalog');
    }
};

// Actions untuk form update ingredient catalog
export const actions: Actions = {
	default: async ({ request, params, locals, cookies }) => {
        // Pastikan user sudah login
        const session = await locals.getSession();
        if (!session) {
            throw redirect(302, '/login');
        }

        const { uuid } = params;
        const formData = await request.formData();
        const name = formData.get('name')?.toString();
        const description = formData.get('description')?.toString();
		const baseUomUuid = formData.get('baseUomUuid')?.toString();
		const minimumStock = formData.get('minimumStock')?.toString();
        const shelfLifeDays = formData.get('shelfLifeDays')?.toString();

        // Validasi input
        const errors: Record<string, string> = {};
        
        if (!name || name.trim() === '') {
            errors.name = 'Nama katalog bahan tidak boleh kosong';
        }

        if (!baseUomUuid) {
            errors.baseUomUuid = 'Satuan dasar harus dipilih';
        }

		if (minimumStock && (isNaN(parseFloat(minimumStock)) || parseFloat(minimumStock) < 0)) {
			errors.minimumStock = 'Stok minimum harus berupa angka positif';
        }

        if (shelfLifeDays && (isNaN(parseInt(shelfLifeDays)) || parseInt(shelfLifeDays) < 0)) {
            errors.shelfLifeDays = 'Masa simpan harus berupa angka positif';
        }

		if (Object.keys(errors).length > 0) {
			return fail(400, { errors, name, description, baseUomUuid, minimumStock, shelfLifeDays });
        }

		try {
			const accessToken = cookies.get('access_token');
			const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
			const normalizedMinimumStock = minimumStock?.trim() ? minimumStock : undefined;
			await updateIngredientCatalogApi(
				uuid,
				{
					name,
					description: description || undefined,
					base_uom_uuid: baseUomUuid,
					minimum_stock: normalizedMinimumStock,
					shelf_life_days: shelfLifeDays ? Number.parseInt(shelfLifeDays, 10) : undefined
				},
				cookieStr
			);

            return { success: true };
        } catch (error) {
            console.error('Error updating ingredient catalog:', error);
            return fail(500, { 
                error: 'Gagal memperbarui katalog bahan', 
				name,
				description,
				baseUomUuid,
				minimumStock,
				shelfLifeDays
            });
        }
    }
};
