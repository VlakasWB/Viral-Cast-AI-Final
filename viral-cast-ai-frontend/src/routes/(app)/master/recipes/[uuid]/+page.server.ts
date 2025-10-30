import { error } from '@sveltejs/kit';
import { getRecipeSetApi } from '$lib/services/recipe.js';
import { getRecipeItemsApi } from '$lib/services/recipe.js';
import { getProductApi } from '$lib/services/product.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies }) => {
	const uuid = params.uuid;
	if (!uuid) throw error(400, 'Missing recipe UUID');

	try {
		const accessToken = cookies.get('access_token');
		const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;
		// Get recipe set details
		const recipeResponse = await getRecipeSetApi(uuid, cookieStr);
		if (!recipeResponse.data?.recipe_set) throw error(404, 'Recipe set not found');

		const recipeSet = recipeResponse.data.recipe_set;

		// Get recipe items for this recipe set
		const itemsResponse = await getRecipeItemsApi(
			{ recipe_uuid: uuid, page: 1, limit: 100 },
			cookieStr
		);

		// Get product details
		let product = null;
		try {
			const productResponse = await getProductApi(
				recipeSet.product_uuid,
				cookieStr
			);
			product = productResponse.data?.product || null;
		} catch (err) {
			console.error('Error loading product:', err);
		}

		return {
			recipeSet,
			recipeItems: itemsResponse.data?.recipe_items || [],
			product
		};
	} catch (err) {
		console.error('Error loading recipe set:', err);
		throw error(404, 'Recipe set not found');
	}
};
