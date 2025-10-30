import type { ApiResponse } from '$lib/types';

// Get API base URL - fallback for client-side
const getApiBaseUrl = () => {
	try {
		// Try to import from server environment
		return process.env.API_BASE_URL || 'http://localhost:12000';
	} catch {
		// Fallback for client-side
		return 'http://localhost:12000';
	}
};

export interface OrderItem {
	product_uuid: string;
	qty: number;
	unit_price: number;
	unit_cost: number;
	line_total: number;
}

export interface CreateOrderRequest {
	order_no: string;
	cashier_uuid: string;
	subtotal: number;
	discount: number;
	tax: number;
	total: number;
	net_profit: number;
	items: OrderItem[];
}

export interface OrderItemResponse {
	uuid: string;
	order_uuid: string;
	product_uuid: string;
	qty: string;
	unit_price: string;
	unit_cost: string;
	line_total: string;
	created_at: number;
	updated_at: number;
	product_name: string;
	product_sku: string;
	product_price: string;
}

export interface OrderResponse {
	uuid: string;
	order_no: string;
	cashier_uuid: string;
	status: string;
	subtotal: string;
	discount: string;
	tax: string;
	total: string;
	net_profit: string;
	created_at: number;
	updated_at: number;
	items?: OrderItemResponse[];
	items_count?: number;
}

export interface OrdersListResponse {
	orders: OrderResponse[];
	total: number;
	page: number;
	limit: number;
	total_pages: number;
}

export interface OrderStatsResponse {
	total_orders: number;
	total_revenue: string;
	total_profit: string;
	avg_order_value: string;
	orders_by_status: any[];
	daily_stats: any[];
}

// Create a new order
export async function createOrderApi(
	orderData: CreateOrderRequest,
	accessToken?: string
): Promise<ApiResponse<OrderResponse>> {
	const API_BASE_URL = getApiBaseUrl();
	const headers: Record<string, string> = {
		'Content-Type': 'application/json'
	};

	// Add authorization header if token is provided
	if (accessToken) {
		headers['Authorization'] = `Bearer ${accessToken}`;
	}

	const response = await fetch(`${API_BASE_URL}/api/v1/orders`, {
		method: 'POST',
		headers,
		body: JSON.stringify(orderData)
	});

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return await response.json();
}

// Get orders list with pagination and filters
export async function getOrdersApi(
	page: number = 1,
	limit: number = 10,
	status?: string
): Promise<ApiResponse<OrdersListResponse>> {
	const API_BASE_URL = getApiBaseUrl();
	const params = new URLSearchParams({
		page: page.toString(),
		limit: limit.toString()
	});

	if (status) {
		params.append('status', status);
	}

	const response = await fetch(`${API_BASE_URL}/api/v1/orders?${params.toString()}`);

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return await response.json();
}

// Get single order by UUID
export async function getOrderByIdApi(orderUuid: string): Promise<ApiResponse<OrderResponse>> {
	const API_BASE_URL = getApiBaseUrl();
	const response = await fetch(`${API_BASE_URL}/api/v1/orders/${orderUuid}`);

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return await response.json();
}

// Update order
export async function updateOrderApi(
	orderUuid: string,
	orderData: Partial<CreateOrderRequest>
): Promise<ApiResponse<OrderResponse>> {
	const API_BASE_URL = getApiBaseUrl();
	const response = await fetch(`${API_BASE_URL}/api/v1/orders/${orderUuid}`, {
		method: 'PUT',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(orderData)
	});

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return await response.json();
}

// Update order status
export async function updateOrderStatusApi(
	orderUuid: string,
	status: string,
	accessToken?: string
): Promise<ApiResponse<OrderResponse>> {
	const API_BASE_URL = getApiBaseUrl();
	const headers: Record<string, string> = {
		'Content-Type': 'application/json'
	};

	// Add authorization header if token is provided
	if (accessToken) {
		headers['Authorization'] = `Bearer ${accessToken}`;
	}

	const response = await fetch(`${API_BASE_URL}/api/v1/orders/${orderUuid}/status`, {
		method: 'PATCH',
		headers,
		body: JSON.stringify({ status })
	});

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return await response.json();
}

// Delete order
export async function deleteOrderApi(orderUuid: string): Promise<ApiResponse<any>> {
	const API_BASE_URL = getApiBaseUrl();
	const response = await fetch(`${API_BASE_URL}/api/v1/orders/${orderUuid}`, {
		method: 'DELETE'
	});

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return await response.json();
}

// Get order statistics
export async function getOrderStatsApi(
	dateFrom?: number,
	dateTo?: number
): Promise<ApiResponse<OrderStatsResponse>> {
	const API_BASE_URL = getApiBaseUrl();
	const params = new URLSearchParams();

	if (dateFrom) {
		params.append('date_from', dateFrom.toString());
	}

	if (dateTo) {
		params.append('date_to', dateTo.toString());
	}

	const response = await fetch(`${API_BASE_URL}/api/v1/orders/stats?${params.toString()}`);

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return await response.json();
}

// Generate order number
export function generateOrderNumber(): string {
	const randomInt = Math.floor(Math.random() * 1000);
	return `ORD-${randomInt}`;
}
