<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import type { Product } from '$lib/types/product.js';
	import { buildImageUrl } from '$lib/utils/imageUrl.js';
	import { cartActions } from '$lib/stores/cartStore';

	let {
		data
	}: {
		data: {
			products: Product[];
		};
	} = $props();

	// Format price to currency
	function formatPrice(price: string): string {
		const numPrice = parseFloat(price);
		return new Intl.NumberFormat('id-ID', {
			style: 'currency',
			currency: 'IDR',
			minimumFractionDigits: 0
		}).format(numPrice);
	}

	// Get complete image URL
	function getImageUrl(imageUrl: string | null): string {
		if (!imageUrl) {
			return 'https://images.unsplash.com/photo-1546069901-ba9599a7e63c?w=400&h=300&fit=crop';
		}
		return buildImageUrl(imageUrl) || imageUrl;
	}

	// Handle add to cart
	function handleAddToCart(product: Product) {
		cartActions.addItem(product, 1);
		cartActions.openCart();

		// Show success notification
		const notification = document.createElement('div');
		notification.className =
			'fixed top-4 right-4 bg-green-500 text-white px-6 py-3 rounded-lg shadow-lg z-50 transition-all duration-300';
		notification.textContent = `${product.name} added to cart!`;
		document.body.appendChild(notification);

		// Remove notification after 3 seconds
		setTimeout(() => {
			notification.remove();
		}, 3000);
	}

	// Handle view details
	function handleViewDetails(product: Product) {
		// TODO: Navigate to product detail page
		console.log('View details:', product);
	}
</script>

<svelte:head>
	<title>Our Products</title>
</svelte:head>

<section class="space-y-6">
	<!-- Header -->
	<div class="space-y-2 text-center">
		<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Our Products</h1>
		<p class="text-lg text-gray-600 dark:text-gray-400">
			Discover our amazing collection of premium products
		</p>
	</div>

	<!-- Products Grid -->
	<div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
		{#each data.products as product (product.uuid)}
			<div
				class="group relative overflow-hidden rounded-xl border border-gray-200 bg-white shadow-lg transition-all duration-300 hover:shadow-xl dark:border-gray-700 dark:bg-gray-800"
			>
				<!-- Product Image -->
				<div
					class="relative overflow-hidden rounded-t-xl bg-gray-100 dark:bg-gray-800"
					style="height: 240px;"
				>
					<img
						src={getImageUrl(product.image_url)}
						alt={product.name}
						class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
						loading="lazy"
						style="width: 100%; height: 240px; object-fit: cover;"
					/>
					<!-- Status Badge -->
					{#if product.status === 'ACTIVE'}
						<div class="absolute top-3 left-3">
							<span
								class="inline-flex items-center rounded-full bg-green-100 px-2 py-1 text-xs font-medium text-green-800 dark:bg-green-900 dark:text-green-200"
							>
								Available
							</span>
						</div>
					{/if}
					<!-- Quick View Button -->
					<div
						class="absolute top-3 right-3 opacity-0 transition-opacity duration-300 group-hover:opacity-100"
					>
						<button
							onclick={() => handleViewDetails(product)}
							class="rounded-full bg-white p-2 shadow-lg transition-colors hover:bg-gray-50 dark:bg-gray-800 dark:hover:bg-gray-700"
							title="Quick View"
							aria-label="Quick view {product.name}"
						>
							<svg
								class="h-4 w-4 text-gray-600 dark:text-gray-400"
								fill="none"
								stroke="currentColor"
								viewBox="0 0 24 24"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
								></path>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
								></path>
							</svg>
						</button>
					</div>
				</div>

				<!-- Product Info -->
				<div class="space-y-3 p-4">
					<!-- Product Name -->
					<h3
						class="line-clamp-2 text-lg font-semibold text-gray-900 transition-colors group-hover:text-blue-600 dark:text-white dark:group-hover:text-blue-400"
					>
						{product.name}
					</h3>

					<!-- SKU -->
					<p class="text-sm text-gray-500 dark:text-gray-400">
						SKU: {product.sku}
					</p>

					<!-- Price -->
					<div class="flex items-center justify-between">
						<span class="text-xl font-bold text-gray-900 dark:text-white">
							{formatPrice(product.price)}
						</span>
					</div>

					<!-- Action Buttons -->
					<div class="flex gap-2 pt-2">
						<Button label="Add to Cart" color="emerald" onClick={() => handleAddToCart(product)} />
						<Button label="Details" color="gray" onClick={() => handleViewDetails(product)} />
					</div>
				</div>
			</div>
		{/each}
	</div>

	<!-- Empty State -->
	{#if data.products.length === 0}
		<div class="py-12 text-center">
			<div
				class="mx-auto mb-4 flex h-24 w-24 items-center justify-center rounded-full bg-gray-100 dark:bg-gray-800"
			>
				<svg class="h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
					></path>
				</svg>
			</div>
			<h3 class="mb-2 text-lg font-medium text-gray-900 dark:text-white">No products available</h3>
			<p class="text-gray-500 dark:text-gray-400">Check back later for new products!</p>
		</div>
	{/if}
</section>

<style>
	.line-clamp-2 {
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
</style>
