<script lang="ts">
	export let isVisible = false;
	export let message = 'Processing Payment...';
	export let onComplete: (() => void) | undefined = undefined;

	let progress = 0;
	let currentStep = 0;

	const steps = [
		'Validating payment details...',
		'Processing transaction...',
		'Confirming order...',
		'Payment successful!'
	];

	// Handle animation sequence
	$: if (isVisible) {
		progress = 0;
		currentStep = 0;

		// Simulate progress steps
		const progressInterval = setInterval(() => {
			progress += 2;

			// Update step based on progress
			if (progress >= 25 && currentStep === 0) currentStep = 1;
			else if (progress >= 50 && currentStep === 1) currentStep = 2;
			else if (progress >= 75 && currentStep === 2) currentStep = 3;

			if (progress >= 100) {
				clearInterval(progressInterval);
				setTimeout(() => {
					onComplete?.();
				}, 1500);
			}
		}, 80);
	}
</script>

{#if isVisible}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-gradient-to-br from-blue-900/90 via-purple-900/90 to-indigo-900/90 backdrop-blur-sm"
	>
		<div class="relative">
			<!-- 3D Card Container -->
			<div
				class="perspective-1000 mx-4 w-full max-w-md transform-gpu rounded-3xl bg-white p-8 text-center shadow-2xl dark:bg-gray-800"
			>
				<!-- 3D Floating Elements -->
				<div
					class="animate-float-1 absolute -top-6 -left-6 h-12 w-12 rounded-full bg-gradient-to-r from-blue-400 to-purple-500 opacity-20"
				></div>
				<div
					class="animate-float-2 absolute -top-4 -right-8 h-8 w-8 rounded-full bg-gradient-to-r from-purple-400 to-pink-500 opacity-30"
				></div>
				<div
					class="animate-float-3 absolute -right-4 -bottom-6 h-10 w-10 rounded-full bg-gradient-to-r from-indigo-400 to-blue-500 opacity-25"
				></div>

				<!-- Main 3D Loading Animation -->
				<div class="relative mb-8">
					<div class="preserve-3d animate-spin-3d relative mx-auto h-24 w-24 transform-gpu">
						<!-- 3D Cube -->
						<div class="preserve-3d absolute inset-0 transform-gpu">
							<!-- Front face -->
							<div
								class="translateZ-12 absolute inset-0 flex transform items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500 to-purple-600"
							>
								<svg
									class="h-8 w-8 text-white"
									fill="none"
									stroke="currentColor"
									viewBox="0 0 24 24"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1"
									/>
								</svg>
							</div>
							<!-- Back face -->
							<div
								class="translateZ-12 rotateY-180 absolute inset-0 flex transform items-center justify-center rounded-2xl bg-gradient-to-br from-purple-500 to-pink-600"
							>
								<svg
									class="h-8 w-8 text-white"
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
							<!-- Right face -->
							<div
								class="rotateY-90 translateZ-12 absolute inset-0 flex transform items-center justify-center rounded-2xl bg-gradient-to-br from-indigo-500 to-blue-600"
							>
								<svg
									class="h-8 w-8 text-white"
									fill="none"
									stroke="currentColor"
									viewBox="0 0 24 24"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
									/>
								</svg>
							</div>
							<!-- Left face -->
							<div
								class="rotateY-90 translateZ-12 absolute inset-0 flex transform items-center justify-center rounded-2xl bg-gradient-to-br from-pink-500 to-red-600"
							>
								<svg
									class="h-8 w-8 text-white"
									fill="none"
									stroke="currentColor"
									viewBox="0 0 24 24"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M13 10V3L4 14h7v7l9-11h-7z"
									/>
								</svg>
							</div>
							<!-- Top face -->
							<div
								class="rotateX-90 translateZ-12 absolute inset-0 transform rounded-2xl bg-gradient-to-br from-yellow-500 to-orange-600"
							></div>
							<!-- Bottom face -->
							<div
								class="rotateX-90 translateZ-12 absolute inset-0 transform rounded-2xl bg-gradient-to-br from-green-500 to-teal-600"
							></div>
						</div>
					</div>

					<!-- Orbiting particles -->
					<div class="animate-spin-slow absolute inset-0">
						<div
							class="absolute top-0 left-1/2 h-2 w-2 -translate-x-1/2 -translate-y-12 transform animate-pulse rounded-full bg-blue-400"
						></div>
						<div
							class="absolute bottom-0 left-1/2 h-2 w-2 -translate-x-1/2 translate-y-12 transform animate-pulse rounded-full bg-purple-400"
						></div>
						<div
							class="absolute top-1/2 left-0 h-2 w-2 -translate-x-12 -translate-y-1/2 transform animate-pulse rounded-full bg-pink-400"
						></div>
						<div
							class="absolute top-1/2 right-0 h-2 w-2 translate-x-12 -translate-y-1/2 transform animate-pulse rounded-full bg-indigo-400"
						></div>
					</div>
				</div>

				<!-- Progress Bar -->
				<div class="mb-6">
					<div class="h-2 w-full overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
						<div
							class="h-full rounded-full bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 transition-all duration-300 ease-out"
							style="width: {progress}%"
						></div>
					</div>
					<div class="mt-2 text-sm text-gray-600 dark:text-gray-400">{progress}%</div>
				</div>

				<!-- Status Text -->
				<h3 class="mb-2 animate-pulse text-xl font-bold text-gray-900 dark:text-white">
					{message}
				</h3>
				<p class="animate-fade-in text-sm text-gray-600 dark:text-gray-400">
					{steps[currentStep]}
				</p>

				<!-- Animated dots -->
				<div class="mt-4 flex justify-center space-x-1">
					{#each Array(3) as _, i}
						<div
							class="h-2 w-2 animate-bounce rounded-full bg-blue-500"
							style="animation-delay: {i * 0.2}s"
						></div>
					{/each}
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.perspective-1000 {
		perspective: 1000px;
	}

	.preserve-3d {
		transform-style: preserve-3d;
	}

	.translateZ-12 {
		transform: translateZ(48px);
	}

	.rotateY-180 {
		transform: rotateY(180deg) translateZ(48px);
	}

	.rotateY-90 {
		transform: rotateY(90deg) translateZ(48px);
	}

	.rotateX-90 {
		transform: rotateX(90deg) translateZ(48px);
	}

	@keyframes spin-3d {
		0% {
			transform: rotateX(0deg) rotateY(0deg);
		}
		25% {
			transform: rotateX(90deg) rotateY(0deg);
		}
		50% {
			transform: rotateX(90deg) rotateY(90deg);
		}
		75% {
			transform: rotateX(0deg) rotateY(90deg);
		}
		100% {
			transform: rotateX(0deg) rotateY(180deg);
		}
	}

	@keyframes float-1 {
		0%,
		100% {
			transform: translateY(0px) rotate(0deg);
		}
		50% {
			transform: translateY(-20px) rotate(180deg);
		}
	}

	@keyframes float-2 {
		0%,
		100% {
			transform: translateY(0px) rotate(0deg);
		}
		50% {
			transform: translateY(-15px) rotate(-180deg);
		}
	}

	@keyframes float-3 {
		0%,
		100% {
			transform: translateY(0px) rotate(0deg);
		}
		50% {
			transform: translateY(-25px) rotate(90deg);
		}
	}

	@keyframes fade-in {
		0% {
			opacity: 0;
			transform: translateY(10px);
		}
		100% {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.animate-spin-3d {
		animation: spin-3d 3s linear infinite;
	}

	.animate-spin-slow {
		animation: spin 8s linear infinite;
	}

	.animate-float-1 {
		animation: float-1 6s ease-in-out infinite;
	}

	.animate-float-2 {
		animation: float-2 4s ease-in-out infinite;
	}

	.animate-float-3 {
		animation: float-3 5s ease-in-out infinite;
	}

	.animate-fade-in {
		animation: fade-in 0.5s ease-out;
	}
</style>
