import { error, redirect } from '@sveltejs/kit';
import { getOrderByIdApi } from '$lib/services/orderApi';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, locals }) => {
	// Authentication is handled by hooks.server.ts
	// If we reach here, user is already authenticated
	const userUuid = locals.user?.id;

	const orderUuid = params.uuid;

	try {
		// Fetch order details from API
		const response = await getOrderByIdApi(orderUuid);

		if (response.code === 200) {
			return {
				order: response.data,
				userUuid
			};
		} else {
			throw error(404, 'Order not found');
		}
	} catch (apiError) {
		console.error('Error loading order:', apiError);

		// Return dummy data for development
		const dummyOrder = {
			uuid: orderUuid,
			order_no: 'ORD-001',
			cashier_uuid: userUuid,
			status: 'PAID',
			subtotal: '150000.00',
			discount: '15000.00',
			tax: '13500.00',
			total: '148500.00',
			net_profit: '50000.00',
			created_at: Date.now() - 86400000, // 1 day ago
			updated_at: Date.now() - 86400000,
			items: [
				{
					uuid: '1',
					order_uuid: orderUuid,
					product_uuid: 'e1648f01-f098-4c31-a773-65e7ec4e1aeb',
					qty: '2.0000',
					unit_price: '85000.00',
					unit_cost: '45000.00',
					line_total: '170000.00',
					created_at: Date.now() - 86400000,
					updated_at: Date.now() - 86400000,
					product_name: 'Premium Coffee Blend',
					product_sku: 'PCB-003',
					product_price: '85000.00'
				},
				{
					uuid: '2',
					order_uuid: orderUuid,
					product_uuid: '36af1081-52eb-4d31-ad74-2b6cabb9ab3e',
					qty: '1.0000',
					unit_price: '25000.00',
					unit_cost: '15000.00',
					line_total: '25000.00',
					created_at: Date.now() - 86400000,
					updated_at: Date.now() - 86400000,
					product_name: 'Chocolate Croissant',
					product_sku: 'CC-001',
					product_price: '25000.00'
				}
			]
		};

		return {
			order: dummyOrder,
			userUuid,
			error: 'Using dummy data - API not available'
		};
	}
};
