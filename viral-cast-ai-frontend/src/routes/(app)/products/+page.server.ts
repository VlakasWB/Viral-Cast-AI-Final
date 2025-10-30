import type { PageServerLoad } from './$types';
import { getProductsApi } from '$lib/services/product.js';
import type { Product } from '$lib/types/product.js';

// Dummy products for display (in real app, this would come from API)
const dummyProducts: Product[] = [
	{
		uuid: '9d9246b5-85cf-42cf-a0f9-a7018eab1cd9',
		name: 'Premium Coffee Blend',
		category_uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		sku: 'PCB-002',
		price: '85000.00',
		current_recipe_uuid: null,
		status: 'ACTIVE',
		image_url: 'https://images.unsplash.com/photo-1447933601403-0c6688de566e?w=400&h=300&fit=crop',
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
		status: 'ACTIVE',
		image_url: 'https://images.unsplash.com/photo-1559056199-641a0ac8b55e?w=400&h=300&fit=crop',
		created_at: Date.now() - 172800000,
		updated_at: Date.now() - 7200000
	},
	{
		uuid: 'f1a2b3c4-d5e6-7890-abcd-ef1234567890',
		name: 'Organic Green Tea',
		category_uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		sku: 'OGT-003',
		price: '45000.00',
		current_recipe_uuid: null,
		status: 'ACTIVE',
		image_url: 'https://images.unsplash.com/photo-1556679343-c7306c1976bc?w=400&h=300&fit=crop',
		created_at: Date.now() - 259200000,
		updated_at: Date.now() - 10800000
	},
	{
		uuid: 'a1b2c3d4-e5f6-7890-1234-567890abcdef',
		name: 'Chocolate Croissant',
		category_uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		sku: 'CC-004',
		price: '25000.00',
		current_recipe_uuid: null,
		status: 'ACTIVE',
		image_url: 'https://images.unsplash.com/photo-1555507036-ab794f4afe5e?w=400&h=300&fit=crop',
		created_at: Date.now() - 345600000,
		updated_at: Date.now() - 14400000
	},
	{
		uuid: 'b2c3d4e5-f6a7-8901-2345-6789abcdef01',
		name: 'Fresh Bagel',
		category_uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		sku: 'FB-005',
		price: '15000.00',
		current_recipe_uuid: null,
		status: 'ACTIVE',
		image_url: 'https://images.unsplash.com/photo-1509440159596-0249088772ff?w=400&h=300&fit=crop',
		created_at: Date.now() - 432000000,
		updated_at: Date.now() - 18000000
	},
	{
		uuid: 'c3d4e5f6-a7b8-9012-3456-789abcdef012',
		name: 'Vanilla Latte',
		category_uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23',
		sku: 'VL-006',
		price: '35000.00',
		current_recipe_uuid: null,
		status: 'ACTIVE',
		image_url: 'https://images.unsplash.com/photo-1461023058943-07fcbe16d735?w=400&h=300&fit=crop',
		created_at: Date.now() - 518400000,
		updated_at: Date.now() - 21600000
	}
];

export const load: PageServerLoad = async ({ cookies }) => {
	try {
		// Try to get products from API
		const token = cookies.get('access_token');
		const cookieStr = token ? `access_token=${token}` : undefined;
		const response = await getProductsApi(
			{
				page: 1,
				limit: 50,
				sort_by: 'name',
				sort_order: 'asc',
				is_active: true
			},
			cookieStr
		);

		// Filter only active products for customer display
		const activeProducts = response.data.products.filter((product) => product.status === 'ACTIVE');

		return {
			products: activeProducts
		};
	} catch (error) {
		console.error('API not available, using dummy data:', error);

		// Return dummy data when API is not available
		return {
			products: dummyProducts
		};
	}
};
