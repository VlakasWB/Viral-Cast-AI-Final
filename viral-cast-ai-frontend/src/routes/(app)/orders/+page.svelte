<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { getOrderByIdApi, type OrderResponse } from '$lib/services/orderApi';

	let { data } = $props();

	// Format price to Indonesian Rupiah
	function formatPrice(price: string | number): string {
		const numPrice = typeof price === 'string' ? parseFloat(price) : price;
		return new Intl.NumberFormat('id-ID', {
			style: 'currency',
			currency: 'IDR',
			minimumFractionDigits: 0
		}).format(numPrice);
	}

	// Format date
	function formatDate(timestamp: number): string {
		return new Date(timestamp).toLocaleDateString('id-ID', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Get status badge color
	function getStatusColor(status: string): string {
		switch (status.toUpperCase()) {
			case 'PAID':
				return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300';
			case 'DRAFT':
				return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300';
			case 'CANCELLED':
				return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300';
			case 'PENDING':
				return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300';
			default:
				return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300';
		}
	}

	// Handle status filter change
	function handleStatusFilter(event: Event) {
		const target = event.target as HTMLSelectElement;
		const status = target.value;
		const url = new URL($page.url);
		if (status === 'all') {
			url.searchParams.delete('status');
		} else {
			url.searchParams.set('status', status);
		}
		url.searchParams.set('page', '1'); // Reset to first page
		goto(url.toString());
	}

	// Handle pagination
	function handlePageChange(newPage: number) {
		const url = new URL($page.url);
		url.searchParams.set('page', newPage.toString());
		goto(url.toString());
	}

	// View order details (open modal instead of new page)
	let isOrderModalOpen = $state(false);
	let selectedOrder: OrderResponse | null = $state(null);
	let isLoadingOrder = $state(false);

	function normalizeOrder(order: any): OrderResponse {
		return {
			uuid: order.uuid,
			order_no: order.order_no ?? '',
			cashier_uuid: order.cashier_uuid ?? '',
			status: order.status ?? 'DRAFT',
			subtotal: order.subtotal ?? '0',
			discount: order.discount ?? '0',
			tax: order.tax ?? '0',
			total: order.total ?? '0',
			net_profit: order.net_profit ?? '0',
			created_at: order.created_at ?? Date.now(),
			updated_at: order.updated_at ?? order.created_at ?? Date.now(),
			items: order.items,
			items_count: order.items_count
		};
	}

	async function viewOrderDetails(orderUuid: string) {
		isOrderModalOpen = true;
		isLoadingOrder = true;
		selectedOrder = null;
		try {
			const res = await getOrderByIdApi(orderUuid);
			if (res?.data) {
				selectedOrder = res.data as OrderResponse;
			}
		} catch (err) {
			// Fallback: gunakan data minimal dari list bila API tidak tersedia
			const fallback = (data.orders || []).find((o: any) => o.uuid === orderUuid);
			selectedOrder = fallback ? normalizeOrder(fallback) : null;
		} finally {
			isLoadingOrder = false;
		}
	}

	function closeOrderModal() {
		isOrderModalOpen = false;
		selectedOrder = null;
	}
</script>

<svelte:head>
	<title>Order History - Viral Cast AI</title>
</svelte:head>

<div class="w-full">
	<!-- Header -->
	<div class="mb-8">
		<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Order History</h1>
		<p class="mt-2 text-gray-600 dark:text-gray-400">View and manage your order history</p>
	</div>

	<!-- Filters -->
	<div
		class="mb-6 rounded-xl border border-gray-200 bg-white p-6 shadow-lg dark:border-gray-700 dark:bg-gray-800"
	>
		<div class="flex flex-wrap items-center gap-4">
			<div class="flex items-center space-x-2">
				<label for="status-filter" class="text-sm font-medium text-gray-700 dark:text-gray-300">Filter by Status:</label>
				<select
					id="status-filter"
					value={data.currentStatus || 'all'}
					onchange={handleStatusFilter}
					class="rounded-lg border border-gray-300 bg-white px-3 py-2 text-gray-900 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
				>
					<option value="all">All Orders</option>
					<option value="DRAFT">Draft</option>
					<option value="PENDING">Pending</option>
					<option value="PAID">Paid</option>
					<option value="CANCELLED">Cancelled</option>
				</select>
			</div>

			<div class="flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-400">
				<span>Total: {data.pagination.total} orders</span>
			</div>
		</div>
	</div>

	<!-- Error Message -->
	{#if data.error}
		<div
			class="mb-6 rounded-lg border border-yellow-200 bg-yellow-50 p-4 dark:border-yellow-700 dark:bg-yellow-900"
		>
			<div class="flex">
				<svg class="mr-2 h-5 w-5 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
					<path
						fill-rule="evenodd"
						d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
						clip-rule="evenodd"
					/>
				</svg>
				<p class="text-yellow-800 dark:text-yellow-200">{data.error}</p>
			</div>
		</div>
	{/if}

	<!-- Orders List -->
	{#if data.orders.length === 0}
		<!-- Empty State -->
		<div
			class="rounded-xl border border-gray-200 bg-white p-12 text-center shadow-lg dark:border-gray-700 dark:bg-gray-800"
		>
			<svg
				class="mx-auto mb-4 h-16 w-16 text-gray-300 dark:text-gray-600"
				fill="none"
				stroke="currentColor"
				viewBox="0 0 24 24"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
				/>
			</svg>
			<h3 class="mb-2 text-lg font-medium text-gray-900 dark:text-white">No orders found</h3>
			<p class="mb-4 text-gray-500 dark:text-gray-400">
				{#if data.currentStatus}
					No orders with status "{data.currentStatus}" found.
				{:else}
					You haven't placed any orders yet.
				{/if}
			</p>
			<a
				href="/products"
				class="inline-flex items-center rounded-lg bg-blue-600 px-6 py-3 font-medium text-white transition-colors hover:bg-blue-700"
			>
				Start Shopping
			</a>
		</div>
	{:else}
		<!-- Orders Table -->
		<div
			class="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-lg dark:border-gray-700 dark:bg-gray-800"
		>
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-gray-50 dark:bg-gray-700">
						<tr>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-300"
							>
								Order
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-300"
							>
								Date
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-300"
							>
								Status
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-300"
							>
								Items
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-300"
							>
								Total
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-300"
							>
								Actions
							</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-gray-200 bg-white dark:divide-gray-700 dark:bg-gray-800">
						{#each data.orders as order (order.uuid)}
							<tr class="transition-colors hover:bg-gray-50 dark:hover:bg-gray-700">
								<td class="px-6 py-4 whitespace-nowrap">
									<div>
										<div class="text-sm font-medium text-gray-900 dark:text-white">
											{order.order_no}
										</div>
										<div class="text-sm text-gray-500 dark:text-gray-400">
											ID: {order.uuid.substring(0, 8)}...
										</div>
									</div>
								</td>
								<td class="px-6 py-4 text-sm whitespace-nowrap text-gray-900 dark:text-white">
									{formatDate(order.created_at)}
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<span
										class="inline-flex rounded-full px-2 py-1 text-xs font-semibold {getStatusColor(
											order.status
										)}"
									>
										{order.status}
									</span>
								</td>
								<td class="px-6 py-4 text-sm whitespace-nowrap text-gray-900 dark:text-white">
									{order.items_count} item{order.items_count !== 1 ? 's' : ''}
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<div class="text-sm font-medium text-gray-900 dark:text-white">
										{formatPrice(order.total)}
									</div>
									{#if order.net_profit}
										<div class="text-sm text-green-600 dark:text-green-400">
											Profit: {formatPrice(order.net_profit)}
										</div>
									{/if}
								</td>
								<td class="px-6 py-4 text-sm font-medium whitespace-nowrap">
						<button
							onclick={() => viewOrderDetails(order.uuid)}
							class="text-blue-600 transition-colors hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-300"
						>
										View Details
									</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>

		<!-- Pagination -->
		{#if data.pagination.total_pages > 1}
			<div class="mt-6 flex items-center justify-between">
				<div class="text-sm text-gray-700 dark:text-gray-300">
					Showing page {data.pagination.page} of {data.pagination.total_pages}
					({data.pagination.total} total orders)
				</div>

				<div class="flex items-center space-x-2">
					<button
						onclick={() => handlePageChange(data.pagination.page - 1)}
						disabled={data.pagination.page <= 1}
						class="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
					>
						Previous
					</button>

					{#each Array.from({ length: Math.min(5, data.pagination.total_pages) }, (_, i) => i + Math.max(1, data.pagination.page - 2)) as pageNum}
						{#if pageNum <= data.pagination.total_pages}
							<button
								onclick={() => handlePageChange(pageNum)}
								class="rounded-lg px-3 py-2 text-sm font-medium {pageNum === data.pagination.page
									? 'bg-blue-600 text-white'
									: 'border border-gray-300 bg-white text-gray-500 hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700'}"
							>
								{pageNum}
							</button>
						{/if}
					{/each}

					<button
						onclick={() => handlePageChange(data.pagination.page + 1)}
						disabled={data.pagination.page >= data.pagination.total_pages}
						class="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
					>
						Next
					</button>
				</div>
			</div>
		{/if}
	{/if}

	<!-- Order Details Modal -->
	{#if isOrderModalOpen}
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
			role="dialog"
			aria-modal="true"
			aria-labelledby="order-modal-title"
			onclick={(e) => e.currentTarget === e.target && closeOrderModal()}
		>
			<div class="w-full max-w-3xl overflow-hidden rounded-2xl bg-white shadow-2xl dark:bg-gray-900">
				<div class="flex items-center justify-between border-b border-gray-200 p-4 dark:border-gray-700">
					<h2 id="order-modal-title" class="text-lg font-semibold text-gray-900 dark:text-white">
						Order Details
					</h2>
					<button
						class="rounded p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
						aria-label="Close"
						onclick={closeOrderModal}
					>
						<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
						</svg>
					</button>
				</div>

				<div class="p-4">
					{#if isLoadingOrder}
						<div class="py-10 text-center text-sm text-gray-600 dark:text-gray-300">Loading order…</div>
					{:else if selectedOrder}
						<div class="space-y-4">
							<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
								<div>
									<div class="text-sm text-gray-500 dark:text-gray-400">Order No</div>
									<div class="font-medium text-gray-900 dark:text-white">{selectedOrder.order_no}</div>
								</div>
								<div>
									<div class="text-sm text-gray-500 dark:text-gray-400">Status</div>
									<span class="inline-flex rounded-full px-2 py-1 text-xs font-semibold {getStatusColor(selectedOrder.status)}">
										{selectedOrder.status}
									</span>
								</div>
								<div>
									<div class="text-sm text-gray-500 dark:text-gray-400">Created</div>
									<div class="font-medium text-gray-900 dark:text-white">{formatDate(selectedOrder.created_at)}</div>
								</div>
								<div>
									<div class="text-sm text-gray-500 dark:text-gray-400">Total</div>
									<div class="font-medium text-gray-900 dark:text-white">{formatPrice(selectedOrder.total)}</div>
								</div>
							</div>

							{#if selectedOrder.items && selectedOrder.items.length > 0}
								<div class="mt-4">
									<h3 class="mb-2 text-sm font-semibold text-gray-900 dark:text-white">Items</h3>
									<ul class="divide-y divide-gray-200 rounded-lg border border-gray-200 dark:divide-gray-700 dark:border-gray-700">
										{#each selectedOrder.items as it (it.uuid)}
											<li class="flex items-center justify-between p-3 text-sm">
												<div>
													<div class="font-medium text-gray-900 dark:text-white">{it.product_name}</div>
													<div class="text-gray-500 dark:text-gray-400">Qty: {it.qty} • Price: {formatPrice(it.unit_price)}</div>
												</div>
												<div class="font-medium text-gray-900 dark:text-white">{formatPrice(it.line_total)}</div>
											</li>
										{/each}
									</ul>
								</div>
							{:else}
								<div class="mt-2 text-sm text-gray-600 dark:text-gray-300">No item details available.</div>
							{/if}
						</div>
					{:else}
						<div class="py-10 text-center text-sm text-gray-600 dark:text-gray-300">Order not found.</div>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</div>
