import { error, redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getProductApi, updateProductApi, uploadProductImageApi } from '$lib/services/product.js';

export const load: PageServerLoad = async ({ params, cookies }) => {
	const uuid = params.uuid;
	if (!uuid) throw error(400, 'Missing product UUID');

	try {
		const token = cookies.get('access_token');
		const cookieStr = token ? `access_token=${token}` : undefined;
		const response = await getProductApi(uuid, cookieStr);
		if (!response.data.product) throw error(404, 'Product not found');
		return { product: response.data.product };
	} catch (err) {
		console.error('Error loading product:', err);
		throw error(404, 'Product not found');
	}
};

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const form = await request.formData();
		const uuid = String(form.get('uuid') ?? '');
		const name = String(form.get('name') ?? '').trim();
		const category_uuid = String(form.get('category_uuid') ?? '').trim();
		const sku = String(form.get('sku') ?? '').trim();
		const price = Number(form.get('price') ?? 0);
		const status = String(form.get('status') ?? 'ACTIVE') as 'ACTIVE' | 'INACTIVE';
		const imageFile = form.get('image') as File | null;
		const existing_image_url = String(form.get('existing_image_url') ?? '').trim();

		if (!uuid) throw error(400, 'Missing product UUID');

		// Validation
		if (!name) {
			return fail(400, {
				errors: { name: 'Product name is required' },
				values: { name, category_uuid, sku, price, status }
			});
		}

		if (!category_uuid) {
			return fail(400, {
				errors: { category_uuid: 'Category is required' },
				values: { name, category_uuid, sku, price, status }
			});
		}

		if (!sku) {
			return fail(400, {
				errors: { sku: 'SKU is required' },
				values: { name, category_uuid, sku, price, status }
			});
		}

		if (price <= 0) {
			return fail(400, {
				errors: { price: 'Price must be greater than 0' },
				values: { name, category_uuid, sku, price, status }
			});
		}

		let image_url = existing_image_url;

		try {
			// Upload new image if provided
			if (imageFile && imageFile.size > 0) {
				try {
					const token = cookies.get('access_token');
					const cookieStr = token ? `access_token=${token}` : undefined;
					const uploadResponse = await uploadProductImageApi(imageFile, cookieStr);
					image_url = uploadResponse.data.image_url;
				} catch (uploadError) {
					console.error('Image upload failed:', uploadError);
					// Continue with existing image if upload fails
				}
			}

			// Update product via API
			const updateData = {
				name,
				category_uuid,
				sku,
				price,
				status,
				current_recipe_uuid: null,
				...(image_url && { image_url })
			};

			const token = cookies.get('access_token');
			const cookieStr = token ? `access_token=${token}` : undefined;
			await updateProductApi(uuid, updateData, cookieStr);
		} catch (error) {
			console.error('Error updating product:', error);
			return fail(500, {
				errors: { general: 'Failed to update product' },
				values: { name, category_uuid, sku, price, status }
			});
		}

		throw redirect(303, '/master/products');
	}
};
