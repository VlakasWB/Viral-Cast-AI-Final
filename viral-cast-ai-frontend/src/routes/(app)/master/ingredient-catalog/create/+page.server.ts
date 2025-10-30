import { redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { createIngredientCatalogApi } from '$lib/services/ingredient-catalog.js';
import { getUOMsApi } from '$lib/services/uom.js';

// Load data untuk form pembuatan ingredient catalog
export const load: PageServerLoad = async ({ locals, cookies, fetch }) => {
    // Pastikan user sudah login
    const session = await locals.getSession();
    if (!session) {
        throw redirect(302, '/login');
    }

    try {
        // Ambil data UOM untuk dropdown
        const accessToken = cookies.get('access_token');
        const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
        const uomsResponse = await getUOMsApi(cookieStr, fetch);
        return {
            uoms: uomsResponse.data ?? []
        };
    } catch (error) {
        console.error('Error loading UOMs:', error);
        return {
            uoms: []
        };
    }
};

// Actions untuk form pembuatan ingredient catalog
export const actions: Actions = {
	default: async ({ request, locals, cookies }) => {
        // Pastikan user sudah login
        const session = await locals.getSession();
        if (!session) {
            throw redirect(302, '/login');
        }

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
            // Kirim data ke API
			const accessToken = cookies.get('access_token');
			const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
			const normalizedMinimumStock = minimumStock?.trim() ? minimumStock : undefined;

			await createIngredientCatalogApi(
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
            console.error('Error creating ingredient catalog:', error);
            return fail(500, { 
                error: 'Gagal membuat katalog bahan', 
				name,
				description,
				baseUomUuid,
				minimumStock,
				shelfLifeDays
            });
        }
    }
};
