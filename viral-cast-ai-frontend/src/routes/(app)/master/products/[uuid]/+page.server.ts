import { error } from '@sveltejs/kit';
import { getProductApi } from '$lib/services/product.js';
import { getRecipeSetsApi } from '$lib/services/recipe.js';
import type { RecipeSet } from '$lib/types/recipe.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies }) => {
	const uuid = params.uuid;
	if (!uuid) throw error(400, 'Missing product UUID');

	try {
		const accessToken = cookies.get('access_token');
		const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
		// Get product details
		const productResponse = await getProductApi(uuid, cookieStr);
		if (!productResponse.data?.product) throw error(404, 'Product not found');

		const product = productResponse.data.product;

		// Get recipe sets for this product
		let recipeSets: RecipeSet[] = [];
		try {
			const recipesResponse = await getRecipeSetsApi(
				{ product_uuid: uuid, page: 1, limit: 100 },
				cookieStr
			);
			recipeSets = recipesResponse.data?.recipe_sets || [];
		} catch (err) {
			console.error('Error loading recipe sets:', err);
		}

		return {
			product,
			recipeSets
		};
	} catch (err) {
		console.error('Error loading product:', err);
		throw error(404, 'Product not found');
	}
};
