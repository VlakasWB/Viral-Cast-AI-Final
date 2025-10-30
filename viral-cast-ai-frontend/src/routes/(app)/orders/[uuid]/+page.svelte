<script lang="ts">
	import { goto } from '$app/navigation';
	import { buildImageUrl } from '$lib/utils/imageUrl';

	export let data;

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
			month: 'long',
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

	// Print order
	function printOrder() {
		window.print();
	}

	// Go back to orders list
	function goBack() {
		goto('/orders');
	}
</script>

<svelte:head>
	<title>Order {data.order.order_no} - Viral Cast AI</title>
</svelte:head>

<div class="w-full">
	<!-- Header -->
	<div class="mb-8 flex items-center justify-between">
		<div class="flex items-center space-x-4">
			<button
				on:click={goBack}
				class="p-2 text-gray-400 transition-colors hover:text-gray-600 dark:hover:text-gray-300"
				aria-label="Go back"
			>
				<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M15 19l-7-7 7-7"
					/>
				</svg>
			</button>
			<div>
				<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Order Details</h1>
				<p class="mt-1 text-gray-600 dark:text-gray-400">Order #{data.order.order_no}</p>
			</div>
		</div>

		<div class="flex items-center space-x-3">
			<button
				on:click={printOrder}
				class="rounded-lg bg-gray-200 px-4 py-2 font-medium text-gray-900 transition-colors hover:bg-gray-300 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600"
			>
				<svg class="mr-2 inline h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"
					/>
				</svg>
				Print
			</button>
			<span
				class="inline-flex rounded-full px-3 py-1 text-sm font-semibold {getStatusColor(
					data.order.status
				)}"
			>
				{data.order.status}
			</span>
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

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
		<!-- Order Information -->
		<div class="space-y-6 lg:col-span-2">
			<!-- Order Summary -->
			<div
				class="rounded-xl border border-gray-200 bg-white p-6 shadow-lg dark:border-gray-700 dark:bg-gray-800"
			>
				<h2 class="mb-4 text-xl font-semibold text-gray-900 dark:text-white">Order Information</h2>

				<div class="grid grid-cols-2 gap-4 text-sm">
					<div>
						<span class="text-gray-500 dark:text-gray-400">Order Number:</span>
						<p class="font-medium text-gray-900 dark:text-white">{data.order.order_no}</p>
					</div>
					<div>
						<span class="text-gray-500 dark:text-gray-400">Order ID:</span>
						<p class="font-mono text-xs font-medium text-gray-900 dark:text-white">
							{data.order.uuid}
						</p>
					</div>
					<div>
						<span class="text-gray-500 dark:text-gray-400">Created:</span>
						<p class="font-medium text-gray-900 dark:text-white">
							{formatDate(data.order.created_at)}
						</p>
					</div>
					<div>
						<span class="text-gray-500 dark:text-gray-400">Last Updated:</span>
						<p class="font-medium text-gray-900 dark:text-white">
							{formatDate(data.order.updated_at)}
						</p>
					</div>
					<div>
						<span class="text-gray-500 dark:text-gray-400">Status:</span>
						<span
							class="inline-flex rounded-full px-2 py-1 text-xs font-semibold {getStatusColor(
								data.order.status
							)}"
						>
							{data.order.status}
						</span>
					</div>
					<div>
						<span class="text-gray-500 dark:text-gray-400">Items Count:</span>
						<p class="font-medium text-gray-900 dark:text-white">
							{data.order.items?.length || 0} items
						</p>
					</div>
				</div>
			</div>

			<!-- Order Items -->
			<div
				class="rounded-xl border border-gray-200 bg-white p-6 shadow-lg dark:border-gray-700 dark:bg-gray-800"
			>
				<h2 class="mb-6 text-xl font-semibold text-gray-900 dark:text-white">Order Items</h2>

				{#if data.order.items && data.order.items.length > 0}
					<div class="space-y-4">
						{#each data.order.items as item (item.uuid)}
							<div class="flex items-center space-x-4 rounded-lg bg-gray-50 p-4 dark:bg-gray-700">
								<!-- Product Image Placeholder -->
								<div class="flex-shrink-0">
									<div
										class="flex h-16 w-16 items-center justify-center rounded-lg bg-gray-200 dark:bg-gray-600"
									>
										<svg
											class="h-8 w-8 text-gray-400"
											fill="none"
											stroke="currentColor"
											viewBox="0 0 24 24"
										>
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
											/>
										</svg>
									</div>
								</div>

								<!-- Product Info -->
								<div class="flex-1">
									<h3 class="font-medium text-gray-900 dark:text-white">{item.product_name}</h3>
									<p class="text-sm text-gray-500 dark:text-gray-400">SKU: {item.product_sku}</p>
									<div class="mt-2 flex items-center space-x-4 text-sm">
										<span class="text-gray-600 dark:text-gray-300">
											Price: {formatPrice(item.unit_price)}
										</span>
										<span class="text-gray-600 dark:text-gray-300">
											Qty: {parseFloat(item.qty)}
										</span>
										<span class="text-gray-600 dark:text-gray-300">
											Cost: {formatPrice(item.unit_cost)}
										</span>
									</div>
								</div>

								<!-- Line Total -->
								<div class="text-right">
									<p class="font-semibold text-gray-900 dark:text-white">
										{formatPrice(item.line_total)}
									</p>
									<p class="text-sm text-green-600 dark:text-green-400">
										Profit: {formatPrice(
											(parseFloat(item.unit_price) - parseFloat(item.unit_cost)) *
												parseFloat(item.qty)
										)}
									</p>
								</div>
							</div>
						{/each}
					</div>
				{:else}
					<div class="py-8 text-center">
						<svg
							class="mx-auto mb-4 h-12 w-12 text-gray-300 dark:text-gray-600"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
							/>
						</svg>
						<p class="text-gray-500 dark:text-gray-400">No items in this order</p>
					</div>
				{/if}
			</div>
		</div>

		<!-- Order Summary -->
		<div class="space-y-6">
			<!-- Financial Summary -->
			<div
				class="rounded-xl border border-gray-200 bg-white p-6 shadow-lg dark:border-gray-700 dark:bg-gray-800"
			>
				<h2 class="mb-6 text-xl font-semibold text-gray-900 dark:text-white">Order Summary</h2>

				<div class="space-y-3">
					<div class="flex justify-between text-sm">
						<span class="text-gray-600 dark:text-gray-400">Subtotal:</span>
						<span class="font-medium text-gray-900 dark:text-white"
							>{formatPrice(data.order.subtotal)}</span
						>
					</div>

					{#if parseFloat(data.order.discount) > 0}
						<div class="flex justify-between text-sm">
							<span class="text-gray-600 dark:text-gray-400">Discount:</span>
							<span class="font-medium text-green-600 dark:text-green-400"
								>-{formatPrice(data.order.discount)}</span
							>
						</div>
					{/if}

					<div class="flex justify-between text-sm">
						<span class="text-gray-600 dark:text-gray-400">Tax:</span>
						<span class="font-medium text-gray-900 dark:text-white"
							>{formatPrice(data.order.tax)}</span
						>
					</div>

					<div class="border-t border-gray-200 pt-3 dark:border-gray-600">
						<div class="flex justify-between">
							<span class="text-lg font-semibold text-gray-900 dark:text-white">Total:</span>
							<span class="text-lg font-bold text-gray-900 dark:text-white"
								>{formatPrice(data.order.total)}</span
							>
						</div>
					</div>

					{#if parseFloat(data.order.net_profit) > 0}
						<div class="border-t border-gray-200 pt-3 dark:border-gray-600">
							<div class="flex justify-between">
								<span class="text-sm font-medium text-green-600 dark:text-green-400"
									>Net Profit:</span
								>
								<span class="text-sm font-bold text-green-600 dark:text-green-400"
									>{formatPrice(data.order.net_profit)}</span
								>
							</div>
						</div>
					{/if}
				</div>
			</div>

			<!-- Actions -->
			<div
				class="rounded-xl border border-gray-200 bg-white p-6 shadow-lg dark:border-gray-700 dark:bg-gray-800"
			>
				<h2 class="mb-4 text-xl font-semibold text-gray-900 dark:text-white">Actions</h2>

				<div class="space-y-3">
					<button
						on:click={printOrder}
						class="w-full rounded-lg bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-700"
					>
						Print Order
					</button>

					{#if data.order.status === 'DRAFT'}
						<button
							class="w-full rounded-lg bg-green-600 px-4 py-2 font-medium text-white transition-colors hover:bg-green-700"
						>
							Mark as Paid
						</button>
					{/if}

					{#if data.order.status !== 'CANCELLED'}
						<button
							class="w-full rounded-lg bg-red-600 px-4 py-2 font-medium text-white transition-colors hover:bg-red-700"
						>
							Cancel Order
						</button>
					{/if}

					<button
						on:click={goBack}
						class="w-full rounded-lg bg-gray-200 px-4 py-2 font-medium text-gray-900 transition-colors hover:bg-gray-300 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600"
					>
						Back to Orders
					</button>
				</div>
			</div>
		</div>
	</div>
</div>

<style>
    @media print {
        .no-print {
            display: none !important;
        }

        :global(body) {
            background: white !important;
        }

		.bg-gray-800,
		.dark\:bg-gray-800 {
			background: white !important;
		}

		.text-white,
		.dark\:text-white {
			color: black !important;
		}
	}
</style>
