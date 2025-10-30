<script lang="ts">
	import { cartItems, cartSummary, cartActions, isCartOpen } from '$lib/stores/cartStore';
	import { buildImageUrl } from '$lib/utils/imageUrl';
	import { goto } from '$app/navigation';
	import type { Product } from '$lib/types/product';

	// State untuk modal checkout
	let isCheckoutOpen = $state(false);

	// Format price to Indonesian Rupiah
	function formatPrice(price: number): string {
		return new Intl.NumberFormat('id-ID', {
			style: 'currency',
			currency: 'IDR',
			minimumFractionDigits: 0
		}).format(price);
	}

	// Handle checkout
	function handleCheckout() {
		if ($cartItems.length === 0) {
			alert('Your cart is empty!');
			return;
		}

		// Buka modal checkout dan tetap di halaman saat ini
		isCheckoutOpen = true;
	}

	// Tutup modal checkout
	function closeCheckoutModal() {
		isCheckoutOpen = false;
	}

	// Konfirmasi pembayaran dan menuju halaman checkout
	function confirmCheckout() {
		isCheckoutOpen = false;
		goto('/checkout');
	}

	// Handle continue shopping
	function handleContinueShopping() {
		cartActions.closeCart();
	}

	// Handle backdrop click
	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			cartActions.closeCart();
		}
	}
</script>

<!-- Cart Modal Overlay -->
{#if $isCartOpen}
    <div
        class="bg-opacity-50 fixed inset-0 z-50 flex items-center justify-center bg-black p-4 transition-opacity duration-300"
        onclick={handleBackdropClick}
        onkeydown={(e) => e.key === 'Escape' && cartActions.closeCart()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="cart-title"
        tabindex="-1"
    >
		<!-- Cart Modal -->
		<div
			class="animate-slide-in-left max-h-[90vh] w-full max-w-2xl scale-100 transform overflow-hidden rounded-2xl bg-white opacity-100 shadow-2xl transition-all duration-300 ease-out dark:bg-gray-900"
		>
			<!-- Cart Header -->
			<div
				class="flex items-center justify-between border-b border-gray-200 p-6 dark:border-gray-700"
			>
				<h2 id="cart-title" class="text-xl font-semibold text-gray-900 dark:text-white">
					Shopping Cart ({$cartSummary.itemCount})
				</h2>
                <button
                    onclick={cartActions.closeCart}
                    class="p-2 text-gray-400 transition-colors hover:text-gray-600 dark:hover:text-gray-300"
                    aria-label="Close cart"
                >
					<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						/>
					</svg>
				</button>
			</div>

			<!-- Cart Content -->
			<div class="flex h-full flex-col">
				<!-- Cart Items -->
				<div class="flex-1 overflow-y-auto p-6">
					{#if $cartItems.length === 0}
						<!-- Empty Cart -->
						<div class="flex h-full flex-col items-center justify-center text-center">
							<svg
								class="mb-4 h-16 w-16 text-gray-300 dark:text-gray-600"
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
							<h3 class="mb-2 text-lg font-medium text-gray-900 dark:text-white">
								Your cart is empty
							</h3>
							<p class="mb-4 text-gray-500 dark:text-gray-400">Add some products to get started!</p>
                            <button
                                onclick={handleContinueShopping}
                                class="rounded-lg bg-blue-600 px-6 py-2 text-white transition-colors hover:bg-blue-700"
                            >
								Continue Shopping
							</button>
						</div>
					{:else}
						<!-- Cart Items List -->
						<div class="space-y-4">
							{#each $cartItems as item (item.product.uuid)}
								<div class="flex items-center space-x-4 rounded-lg bg-gray-50 p-4 dark:bg-gray-800">
									<!-- Product Image -->
									<div class="flex-shrink-0">
										<img
											src={buildImageUrl(item.product.image_url)}
											alt={item.product.name}
											class="h-16 w-16 rounded-lg object-cover"
										/>
									</div>

									<!-- Product Info -->
									<div class="min-w-0 flex-1">
										<h4 class="truncate text-sm font-medium text-gray-900 dark:text-white">
											{item.product.name}
										</h4>
										<p class="text-sm text-gray-500 dark:text-gray-400">
											{item.product.sku}
										</p>
										<p class="text-sm font-medium text-gray-900 dark:text-white">
											{formatPrice(item.unit_price)}
										</p>
									</div>

									<!-- Quantity Controls -->
									<div class="flex items-center space-x-2">
                                        <button
                                            onclick={() => cartActions.decreaseQuantity(item.product.uuid)}
                                            class="p-1 text-gray-400 transition-colors hover:text-gray-600 dark:hover:text-gray-300"
                                            aria-label="Decrease quantity"
                                        >
											<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path
													stroke-linecap="round"
													stroke-linejoin="round"
													stroke-width="2"
													d="M20 12H4"
												/>
											</svg>
										</button>

										<span
											class="min-w-[3rem] rounded border border-gray-300 bg-white px-3 py-1 text-center text-sm font-medium text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
										>
											{item.quantity}
										</span>

                                        <button
                                            onclick={() => cartActions.increaseQuantity(item.product.uuid)}
                                            class="p-1 text-gray-400 transition-colors hover:text-gray-600 dark:hover:text-gray-300"
                                            aria-label="Increase quantity"
                                        >
											<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path
													stroke-linecap="round"
													stroke-linejoin="round"
													stroke-width="2"
													d="M12 6v6m0 0v6m0-6h6m-6 0H6"
												/>
											</svg>
										</button>
									</div>

									<!-- Remove Button -->
                                    <button
                                        onclick={() => cartActions.removeItem(item.product.uuid)}
                                        class="p-1 text-red-400 transition-colors hover:text-red-600"
                                        aria-label="Remove item"
                                    >
										<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
											/>
										</svg>
									</button>
								</div>

								<!-- Line Total -->
								<div class="text-right text-sm font-medium text-gray-900 dark:text-white">
									Subtotal: {formatPrice(item.line_total)}
								</div>
							{/each}
						</div>
					{/if}
				</div>

				<!-- Cart Footer -->
				{#if $cartItems.length > 0}
					<div class="space-y-4 border-t border-gray-200 p-6 dark:border-gray-700">
						<!-- Cart Summary -->
						<div class="space-y-2">
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
								class="flex justify-between border-t border-gray-200 pt-2 text-lg font-semibold text-gray-900 dark:border-gray-700 dark:text-white"
							>
								<span>Total</span>
								<span>{formatPrice($cartSummary.total)}</span>
							</div>
						</div>

						<!-- Action Buttons -->
						<div class="space-y-3">
                            <button
                                onclick={handleCheckout}
                                class="w-full rounded-lg bg-green-600 px-6 py-3 font-medium text-white transition-colors hover:bg-green-700"
                            >
								Proceed to Checkout
							</button>
                            <button
                                onclick={handleContinueShopping}
                                class="w-full rounded-lg bg-gray-200 px-6 py-3 font-medium text-gray-900 transition-colors hover:bg-gray-300 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600"
                            >
								Continue Shopping
							</button>
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<!-- Checkout Modal -->
{#if isCheckoutOpen && $isCartOpen}
	<div
		class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 p-4"
		role="dialog"
		aria-modal="true"
		aria-labelledby="checkout-title"
		onclick={(e) => e.currentTarget === e.target && closeCheckoutModal()}
		onkeydown={(e) => e.key === 'Escape' && closeCheckoutModal()}
		tabindex="-1"
	>
		<div class="w-full max-w-md overflow-hidden rounded-2xl bg-white shadow-2xl dark:bg-gray-900">
			<div class="flex items-center justify-between border-b border-gray-200 p-4 dark:border-gray-700">
				<h2 id="checkout-title" class="text-lg font-semibold text-gray-900 dark:text-white">Confirm Payment</h2>
				<button
					class="rounded p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
					aria-label="Close"
					onclick={closeCheckoutModal}
				>
					<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>

			<div class="space-y-4 p-4">
				<div class="space-y-2 text-sm text-gray-700 dark:text-gray-300">
					<div class="flex justify-between"><span>Subtotal</span><span>{formatPrice($cartSummary.subtotal)}</span></div>
					<div class="flex justify-between"><span>Tax</span><span>{formatPrice($cartSummary.tax)}</span></div>
					{#if $cartSummary.discount > 0}
						<div class="flex justify-between text-green-600 dark:text-green-400"><span>Discount</span><span>-{formatPrice($cartSummary.discount)}</span></div>
					{/if}
					<div class="flex justify-between border-t border-gray-200 pt-2 text-base font-semibold dark:border-gray-700"><span>Total</span><span>{formatPrice($cartSummary.total)}</span></div>
				</div>

				<div class="flex gap-2 pt-2">
					<button onclick={confirmCheckout} class="flex-1 rounded-lg bg-green-600 px-4 py-2 text-white">Confirm & Pay</button>
					<button onclick={closeCheckoutModal} class="rounded-lg bg-[var(--color-yellow-deep-600)] px-4 py-2 text-white">Cancel</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	@keyframes slide-in-left {
		0% {
			transform: translateX(-100%) scale(0.9);
			opacity: 0;
		}
		100% {
			transform: translateX(0) scale(1);
			opacity: 1;
		}
	}

	@keyframes slide-out-left {
		0% {
			transform: translateX(0) scale(1);
			opacity: 1;
		}
		100% {
			transform: translateX(-100%) scale(0.9);
			opacity: 0;
		}
	}

	:global(.animate-slide-in-left) {
		animation: slide-in-left 0.3s ease-out forwards;
	}

	:global(.animate-slide-out-left) {
		animation: slide-out-left 0.3s ease-in forwards;
	}
</style>
