<script lang="ts">
	export let isVisible = false;
	export let message = 'Order placed successfully!';
	export let onComplete: (() => void) | undefined = undefined;

	let showSuccess = false;
	let showLoading = true;

	// Handle animation sequence
	$: if (isVisible) {
		showLoading = true;
		showSuccess = false;

		// Show success after loading animation
		setTimeout(() => {
			showLoading = false;
			showSuccess = true;

			// Auto hide after success animation and redirect
			setTimeout(() => {
				onComplete?.();
			}, 2500); // Increased time to 2.5 seconds for better UX
		}, 2000);
	}
</script>

{#if isVisible}
	<div class="bg-opacity-50 fixed inset-0 z-50 flex items-center justify-center bg-black">
		<div
			class="mx-4 w-full max-w-sm rounded-2xl bg-white p-8 text-center shadow-2xl dark:bg-gray-800"
		>
			{#if showLoading}
				<!-- Loading Animation -->
				<div class="mb-6">
					<div class="relative">
						<div class="mx-auto h-16 w-16">
							<div
								class="absolute inset-0 rounded-full border-4 border-blue-200 dark:border-blue-800"
							></div>
							<div
								class="absolute inset-0 animate-spin rounded-full border-4 border-blue-600 border-t-transparent"
							></div>
						</div>
					</div>
				</div>
				<h3 class="mb-2 text-lg font-semibold text-gray-900 dark:text-white">Processing Order</h3>
				<p class="text-gray-600 dark:text-gray-400">Please wait while we process your order...</p>
			{:else if showSuccess}
				<!-- Success Animation -->
				<div class="mb-6">
					<div
						class="mx-auto flex h-16 w-16 animate-bounce items-center justify-center rounded-full bg-green-100 dark:bg-green-900"
					>
						<svg
							class="h-8 w-8 text-green-600 dark:text-green-400"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M5 13l4 4L19 7"
							/>
						</svg>
					</div>
				</div>
				<h3 class="mb-2 text-lg font-semibold text-green-600 dark:text-green-400">
					Payment Successful!
				</h3>
				<p class="mb-3 text-gray-600 dark:text-gray-400">{message}</p>
				<p class="text-sm text-gray-500 dark:text-gray-500">Redirecting to menu...</p>
			{/if}
		</div>
	</div>
{/if}

<style>
	@keyframes bounce {
		0%,
		20%,
		53%,
		80%,
		100% {
			transform: translate3d(0, 0, 0);
		}
		40%,
		43% {
			transform: translate3d(0, -30px, 0);
		}
		70% {
			transform: translate3d(0, -15px, 0);
		}
		90% {
			transform: translate3d(0, -4px, 0);
		}
	}

	.animate-bounce {
		animation: bounce 1s ease-in-out;
	}
</style>
