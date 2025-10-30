<script lang="ts">
	import { cartItems, cartSummary, cartActions } from '$lib/stores/cartStore';
	import { buildImageUrl } from '$lib/utils/imageUrl';
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import type { SubmitFunction } from '@sveltejs/kit';

	let { data, form } = $props();

    let isProcessing = $state(false);
    let paymentMethod = $state('cash');

	// Redirect if cart is empty
	onMount(() => {
		if ($cartItems.length === 0) {
			goto('/products');
		}
	});

	// Format price to Indonesian Rupiah
	function formatPrice(price: number): string {
		return new Intl.NumberFormat('id-ID', {
			style: 'currency',
			currency: 'IDR',
			minimumFractionDigits: 0
		}).format(price);
	}

	// Handle form submission
	const handleSubmit: SubmitFunction = async ({ formData }) => {
		if ($cartItems.length === 0) {
			alert('Your cart is empty!');
			return;
		}

		isProcessing = true;

		// Prepare order data
		const orderData = {
			subtotal: $cartSummary.subtotal,
			discount: $cartSummary.discount,
			tax: $cartSummary.tax,
			total: $cartSummary.total,
			net_profit: $cartItems.reduce((sum, item) => {
				const cost = parseFloat(item.product.price) * 0.6; // Assume 40% margin
				const profit = (item.unit_price - cost) * item.quantity;
				return sum + profit;
			}, 0),
			items: $cartItems.map((item) => ({
				product_uuid: item.product.uuid,
				qty: item.quantity,
				unit_price: item.unit_price,
				unit_cost: parseFloat(item.product.price) * 0.6, // Assume 40% margin
				line_total: item.line_total
			})),
			payment_method: paymentMethod
		};

		formData.set('orderData', JSON.stringify(orderData));

		return async ({ result, update }) => {
			if (result.type === 'success') {
				// Clear cart and redirect to success page
				cartActions.clearCart();
				goto('/success');
			} else if (result.type === 'failure') {
				isProcessing = false;
				alert(result.data?.error || 'Failed to place order');
			} else {
				isProcessing = false;
			}

			await update();
		};
	};

	// Handle 3D loading completion - function not used anymore
	function handleLoadingComplete() {
		// This function is kept for compatibility but not used
	}
</script>

<svelte:head>
	<title>Cashier - Process Payment</title>
</svelte:head>

<div class="w-full">
	<div class="mb-8">
		<h1 class="text-3xl font-bold text-gray-900 dark:text-white">💰 Process Payment</h1>
		<p class="mt-2 text-gray-600 dark:text-gray-400">
			Review order items and process customer payment
		</p>
	</div>

	{#if $cartItems.length === 0}
		<!-- Empty Cart Message -->
		<div class="py-12 text-center">
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
					d="M3 3h2l.4 2M7 13h10l4-8H5.4m0 0L7 13m0 0l-1.5 6M7 13l-1.5-6m0 0L4 5M7 13h10m0 0l1.5 6M17 13l1.5-6"
				/>
			</svg>
			<h2 class="mb-2 text-xl font-semibold text-gray-900 dark:text-white">Your cart is empty</h2>
			<p class="mb-4 text-gray-600 dark:text-gray-400">
				Add some products to continue with checkout
			</p>
			<a
				href="/products"
				class="inline-flex items-center rounded-lg bg-blue-600 px-6 py-3 font-medium text-white transition-colors hover:bg-blue-700"
			>
				Continue Shopping
			</a>
		</div>
	{:else}
		<div class="grid grid-cols-1 gap-8 lg:grid-cols-2">
			<!-- Order Summary -->
			<div class="lg:col-span-1">
				<div
					class="rounded-xl border border-gray-200 bg-white p-6 shadow-lg dark:border-gray-700 dark:bg-gray-800"
				>
					<h2 class="mb-6 text-xl font-semibold text-gray-900 dark:text-white">Order Summary</h2>

					<!-- Order Items -->
					<div class="mb-6 space-y-4">
						{#each $cartItems as item (item.product.uuid)}
							<div class="flex items-center space-x-4 rounded-lg bg-gray-50 p-4 dark:bg-gray-700">
								<!-- Product Image -->
								<div class="flex-shrink-0">
									<img
										src={buildImageUrl(item.product.image_url)}
										alt={item.product.name}
										class="h-16 w-16 rounded-lg object-cover"
									/>
								</div>

								<!-- Product Info -->
								<div class="flex-1">
									<h3 class="font-medium text-gray-900 dark:text-white">{item.product.name}</h3>
									<p class="text-sm text-gray-500 dark:text-gray-400">{item.product.sku}</p>
									<p class="text-sm text-gray-600 dark:text-gray-300">
										{formatPrice(item.unit_price)} × {item.quantity}
									</p>
								</div>

								<!-- Line Total -->
								<div class="text-right">
									<p class="font-medium text-gray-900 dark:text-white">
										{formatPrice(item.line_total)}
									</p>
								</div>
							</div>
						{/each}
					</div>

					<!-- Order Totals -->
					<div class="space-y-2 border-t border-gray-200 pt-4 dark:border-gray-600">
						<div class="flex justify-between text-sm text-gray-600 dark:text-gray-400">
							<span>Subtotal</span>
							<span>{formatPrice($cartSummary.subtotal)}</span>
						</div>
						<div class="flex justify-between text-sm text-gray-600 dark:text-gray-400">
							<span>Tax (9%)</span>
							<span>{formatPrice($cartSummary.tax)}</span>
						</div>
						{#if $cartSummary.discount > 0}
							<div class="flex justify-between text-sm text-green-600 dark:text-green-400">
								<span>Discount</span>
								<span>-{formatPrice($cartSummary.discount)}</span>
							</div>
						{/if}
						<div
							class="flex justify-between border-t border-gray-200 pt-2 text-lg font-semibold text-gray-900 dark:border-gray-600 dark:text-white"
						>
							<span>Total</span>
							<span>{formatPrice($cartSummary.total)}</span>
						</div>
					</div>
				</div>
			</div>

			<!-- Payment & Actions -->
			<div class="space-y-6">
				<!-- Payment Information -->
				<div
					class="rounded-xl border border-gray-200 bg-white p-6 shadow-lg dark:border-gray-700 dark:bg-gray-800"
				>
					<h2 class="mb-6 text-xl font-semibold text-gray-900 dark:text-white">Payment Method</h2>

					<form method="POST" action="?/createOrder" use:enhance={handleSubmit} class="space-y-4">
						<!-- Payment Method -->
						<div>
							<label
								for="payment-method"
								class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
							>
								Payment Method *
							</label>
							<div class="space-y-2" id="payment-method">
								<label class="flex items-center">
									<input
										type="radio"
										bind:group={paymentMethod}
										value="cash"
										class="text-blue-600 focus:ring-blue-500"
									/>
									<span class="ml-2 text-gray-700 dark:text-gray-300">Cash</span>
								</label>
								<label class="flex items-center">
									<input
										type="radio"
										bind:group={paymentMethod}
										value="card"
										class="text-blue-600 focus:ring-blue-500"
									/>
									<span class="ml-2 text-gray-700 dark:text-gray-300">Credit/Debit Card</span>
								</label>
								<label class="flex items-center">
									<input
										type="radio"
										bind:group={paymentMethod}
										value="digital_wallet"
										class="text-blue-600 focus:ring-blue-500"
									/>
									<span class="ml-2 text-gray-700 dark:text-gray-300">Digital Wallet</span>
								</label>
							</div>
						</div>

						<!-- Action Buttons -->
						<div class="space-y-3 pt-4">
							<button
								type="submit"
								disabled={isProcessing}
								class="w-full rounded-lg bg-green-600 px-6 py-3 font-medium text-white transition-colors hover:bg-green-700 disabled:cursor-not-allowed disabled:bg-gray-400"
							>
								{#if isProcessing}
									<svg
										class="mr-3 -ml-1 inline h-5 w-5 animate-spin text-white"
										xmlns="http://www.w3.org/2000/svg"
										fill="none"
										viewBox="0 0 24 24"
									>
										<circle
											class="opacity-25"
											cx="12"
											cy="12"
											r="10"
											stroke="currentColor"
											stroke-width="4"
										></circle>
										<path
											class="opacity-75"
											fill="currentColor"
											d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
										></path>
									</svg>
									Processing Order...
								{:else}
									💰 Process Payment - {formatPrice($cartSummary.total)}
								{/if}
							</button>

							<a
								href="/products"
								class="block w-full rounded-lg bg-gray-200 px-6 py-3 text-center font-medium text-gray-900 transition-colors hover:bg-gray-300 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600"
							>
								🛒 Back to Shop
							</a>
						</div>
					</form>
				</div>
			</div>
		</div>
	{/if}
</div>

{#if form?.error}
	<div class="fixed top-4 right-4 z-50 rounded-lg bg-red-500 px-6 py-3 text-white shadow-lg">
		{form.error}
	</div>
{/if}

{#if form?.success}
	<div class="fixed top-4 right-4 z-50 rounded-lg bg-green-500 px-6 py-3 text-white shadow-lg">
		{form.message}
	</div>
{/if}
