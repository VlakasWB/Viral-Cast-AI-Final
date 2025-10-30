import { redirect } from '@sveltejs/kit';
import { getOrdersApi } from '$lib/services/orderApi';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ url, locals }) => {
	// Authentication is handled by hooks.server.ts
	// If we reach here, user is already authenticated
	const userUuid = locals.user?.id;

	// Get query parameters
	const page = parseInt(url.searchParams.get('page') || '1');
	const limit = parseInt(url.searchParams.get('limit') || '10');
	const status = url.searchParams.get('status') || undefined;

	try {
		// Fetch orders from API
		const response = await getOrdersApi(page, limit, status);

		if (response.code === 200) {
			return {
				orders: response.data.orders,
				pagination: {
					total: response.data.total,
					page: response.data.page,
					limit: response.data.limit,
					total_pages: response.data.total_pages
				},
				currentStatus: status,
				userUuid
			};
		} else {
			// If API fails, return empty data
			return {
				orders: [],
				pagination: {
					total: 0,
					page: 1,
					limit: 10,
					total_pages: 0
				},
				currentStatus: status,
				userUuid,
				error: 'Failed to load orders'
			};
		}
	} catch (error) {
		console.error('Error loading orders:', error);

		// Return dummy data for development
		const dummyOrders = [
			{
				uuid: '1',
				order_no: 'ORD-001',
				cashier_uuid: userUuid,
				status: 'PAID',
				total: '148500.00',
				net_profit: '50000.00',
				created_at: Date.now() - 86400000, // 1 day ago
				items_count: 2
			},
			{
				uuid: '2',
				order_no: 'ORD-002',
				cashier_uuid: userUuid,
				status: 'DRAFT',
				total: '85000.00',
				net_profit: '25000.00',
				created_at: Date.now() - 172800000, // 2 days ago
				items_count: 1
			}
		];

		return {
			orders: dummyOrders,
			pagination: {
				total: dummyOrders.length,
				page: 1,
				limit: 10,
				total_pages: 1
			},
			currentStatus: status,
			userUuid,
			error: 'Using dummy data - API not available'
		};
	}
};
