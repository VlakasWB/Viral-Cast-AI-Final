import type { PageServerLoad } from './$types';
import { dev } from '$app/environment';
import { getRecipeSetsApi } from '$lib/services/recipe.js';
import { getProductsApi } from '$lib/services/product.js';

// Dummy products data as fallback
const dummyProducts = [
	{
		uuid: '9d9246b5-85cf-42cf-a0f9-a7018eab1cd9',
		name: 'Premium Coffee Blend',
		category_uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		sku: 'PCB-002',
		price: '85000.00',
		current_recipe_uuid: 'recipe-premium-coffee',
		status: 'ACTIVE' as const,
		image_url: 'https://example.com/images/coffee-blend-1.jpg',
		created_at: Date.now() - 86400000,
		updated_at: Date.now() - 3600000
	},
	{
		uuid: 'd780f3f7-d5b7-457a-ac6c-8450a6a1be3d',
		name: 'Artisan Espresso Beans',
		category_uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		sku: 'AEB-001',
		price: '120000.00',
		current_recipe_uuid: null,
		status: 'ACTIVE' as const,
		image_url: null,
		created_at: Date.now() - 172800000,
		updated_at: Date.now() - 7200000
	},
	{
		uuid: 'f1a2b3c4-d5e6-7890-abcd-ef1234567890',
		name: 'Organic Green Tea',
		category_uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		sku: 'OGT-003',
		price: '45000.00',
		current_recipe_uuid: 'recipe-organic-green-tea',
		status: 'ACTIVE' as const,
		image_url: 'https://example.com/images/green-tea.jpg',
		created_at: Date.now() - 259200000,
		updated_at: Date.now() - 10800000
	},
	{
		uuid: 'a1b2c3d4-e5f6-7890-1234-567890abcdef',
		name: 'Chocolate Croissant',
		category_uuid: '2a8ceed7-1b26-48ac-a191-04d31f254419',
		sku: 'CC-004',
		price: '25000.00',
		current_recipe_uuid: 'recipe-chocolate-croissant',
		status: 'INACTIVE' as const,
		image_url: null,
		created_at: Date.now() - 345600000,
		updated_at: Date.now() - 14400000
	},
	{
		uuid: 'b2c3d4e5-f6a7-8901-2345-6789abcdef01',
		name: 'Fresh Bagel',
		category_uuid: '2a8ceed7-1b26-48ac-a191-04d31f254419',
		sku: 'FB-005',
		price: '15000.00',
		current_recipe_uuid: null,
		status: 'ACTIVE' as const,
		image_url: 'https://example.com/images/bagel.jpg',
		created_at: Date.now() - 432000000,
		updated_at: Date.now() - 18000000
	}
];

// Dummy recipe sets data as fallback
const dummyRecipeSets = [
	{
		uuid: 'recipe-premium-coffee',
		product_uuid: '9d9246b5-85cf-42cf-a0f9-a7018eab1cd9',
		name: 'Premium Coffee Blend Base',
		yield_qty: 12,
		effective_from: Date.now() - 86400000,
		effective_to: Date.now() + 86400000 * 90,
		is_active: true,
		created_at: Date.now() - 86400000,
		updated_at: Date.now() - 3600000
	},
	{
		uuid: 'recipe-organic-green-tea',
		product_uuid: 'f1a2b3c4-d5e6-7890-abcd-ef1234567890',
		name: 'Organic Green Tea Batch',
		yield_qty: 20,
		effective_from: Date.now() - 172800000,
		effective_to: Date.now() + 86400000 * 60,
		is_active: true,
		created_at: Date.now() - 172800000,
		updated_at: Date.now() - 7200000
	},
	{
		uuid: 'recipe-chocolate-croissant',
		product_uuid: 'a1b2c3d4-e5f6-7890-1234-567890abcdef',
		name: 'Chocolate Croissant Dough',
		yield_qty: 48,
		effective_from: Date.now() - 259200000,
		effective_to: Date.now() + 86400000 * 45,
		is_active: false,
		created_at: Date.now() - 259200000,
		updated_at: Date.now() - 10800000
	}
];

export const load: PageServerLoad = async ({ url, cookies }) => {
    const page = Number(url.searchParams.get('page')) || 1;
    const limit = Number(url.searchParams.get('limit')) || 10;
    const product_uuid = url.searchParams.get('product_uuid') || undefined;
    const product_name = url.searchParams.get('name') || undefined;
    const is_active =
        url.searchParams.get('is_active') === 'true'
            ? true
            : url.searchParams.get('is_active') === 'false'
                ? false
                : undefined;
    const search = url.searchParams.get('search') || undefined;

    try {
        const accessToken = cookies.get('access_token');
        const cookieStr = accessToken ? `access_token=${accessToken}` : undefined;

        // Fetch recipe sets first, using product_uuid directly when provided
        console.log('Fetching recipe sets...');
        const recipeResponse = await getRecipeSetsApi(
            {
                page,
                limit,
                product_uuid: product_uuid || undefined,
                name: product_name || undefined,
                is_active,
                search
            },
            cookieStr
        );
        console.log('Recipe sets response:', recipeResponse);

        // Fetch products for display and filters with wider coverage
        console.log('Fetching products...');
        const productsResponse = await getProductsApi(
            { page: 1, limit: 1000, is_active: undefined },
            cookieStr
        );
        console.log('Products response count:', productsResponse?.data?.products?.length ?? 0);

        return {
            recipeSets: recipeResponse.data.recipe_sets || [],
            products: productsResponse.data.products || [],
            pagination: {
                page: recipeResponse.data.pagination?.page || 1,
                limit: recipeResponse.data.pagination?.limit || 10,
                total: recipeResponse.data.pagination?.total || 0,
                total_pages: recipeResponse.data.pagination?.total_pages || 1
            },
            filters: {
                product_uuid,
                product_name: product_name || '',
                is_active,
                search
            }
        };
    } catch (error) {
        console.error('Error loading recipe sets:', error);
        if (!dev) {
            // In non-dev environments, fail closed without dummy data
            return {
                recipeSets: [],
                products: [],
                pagination: {
                    page: 1,
                    limit: 10,
                    total: 0,
                    total_pages: 1
                },
                filters: {
                    product_uuid,
                    product_name: product_name || '',
                    is_active,
                    search
                }
            };
        }
        console.log('Using dummy data as fallback (dev only)...');
		
		// Filter dummy data based on search parameters
		let filteredRecipeSets = dummyRecipeSets;
		
		if (search) {
			filteredRecipeSets = filteredRecipeSets.filter(recipe => 
				recipe.name.toLowerCase().includes(search.toLowerCase())
			);
		}
		
		if (is_active !== undefined) {
			filteredRecipeSets = filteredRecipeSets.filter(recipe => recipe.is_active === is_active);
		}
		
		if (product_uuid) {
			filteredRecipeSets = filteredRecipeSets.filter(recipe => recipe.product_uuid === product_uuid);
		}
		
        return {
            recipeSets: filteredRecipeSets,
            products: dummyProducts,
            pagination: {
                page: 1,
                limit: 10,
                total: filteredRecipeSets.length,
                total_pages: Math.ceil(filteredRecipeSets.length / 10)
            },
            filters: {
                product_uuid,
                product_name: product_name || '',
                is_active,
                search
            }
        };
    }
};
