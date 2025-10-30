<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let showAnimation = false;
	let animationStep = 0;

	onMount(() => {
		// Start animation sequence
		showAnimation = true;

		// Animation steps
		setTimeout(() => (animationStep = 1), 500); // Show 3D elements
		setTimeout(() => (animationStep = 2), 1500); // Show success message
		setTimeout(() => (animationStep = 3), 3000); // Show completion
	});

	function goToMenu() {
		goto('/products');
	}
</script>

<svelte:head>
	<title>Payment Successful</title>
</svelte:head>

<div
	class="flex min-h-screen items-center justify-center overflow-hidden bg-gradient-to-br from-emerald-400 via-blue-500 to-purple-600 p-4"
>
	<!-- Background Particles -->
	<div class="absolute inset-0">
		{#each Array(20) as _, i}
			<div
				class="animate-float absolute h-2 w-2 rounded-full bg-white/20"
				style="
					left: {Math.random() * 100}%; 
					top: {Math.random() * 100}%; 
					animation-delay: {Math.random() * 3}s;
					animation-duration: {3 + Math.random() * 2}s;
				"
			></div>
		{/each}
	</div>

	<!-- Main Success Card -->
	<div
		class="relative z-10 w-full max-w-md rounded-3xl border border-white/20 bg-white/10 p-8 text-center shadow-2xl backdrop-blur-lg"
	>
		<!-- 3D Success Animation -->
		<div class="relative mb-8 flex h-32 items-center justify-center">
			{#if showAnimation}
				<!-- 3D Rotating Success Icon -->
				<div class="preserve-3d animate-spin-3d relative transform-gpu">
					<!-- Main Success Circle -->
					<div
						class="translateZ-12 flex h-24 w-24 transform items-center justify-center rounded-full bg-gradient-to-br from-emerald-400 to-green-500 shadow-lg"
					>
						<svg
							class="animate-bounce-in h-12 w-12 text-white"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="3"
								d="M5 13l4 4L19 7"
							/>
						</svg>
					</div>

					<!-- 3D Ring Elements -->
					<div
						class="rotateX-45 translateZ-8 absolute inset-0 h-24 w-24 transform animate-pulse rounded-full border-4 border-white/30"
					></div>
					<div
						class="rotateY-45 translateZ-8 absolute inset-0 h-24 w-24 transform animate-pulse rounded-full border-4 border-emerald-300/40"
						style="animation-delay: 0.5s"
					></div>

					<!-- Floating Elements -->
					<div
						class="absolute -top-4 -right-4 h-6 w-6 animate-bounce rounded-full bg-yellow-400"
						style="animation-delay: 1s"
					></div>
					<div
						class="absolute -bottom-4 -left-4 h-4 w-4 animate-bounce rounded-full bg-pink-400"
						style="animation-delay: 1.5s"
					></div>
					<div
						class="absolute -top-4 -left-4 h-5 w-5 animate-bounce rounded-full bg-blue-400"
						style="animation-delay: 2s"
					></div>
				</div>

				<!-- Orbiting Particles -->
				<div class="animate-spin-slow absolute inset-0">
					<div
						class="absolute top-0 left-1/2 h-3 w-3 -translate-x-1/2 -translate-y-16 transform animate-pulse rounded-full bg-white/60"
					></div>
					<div
						class="absolute bottom-0 left-1/2 h-3 w-3 -translate-x-1/2 translate-y-16 transform animate-pulse rounded-full bg-white/60"
					></div>
					<div
						class="absolute top-1/2 left-0 h-3 w-3 -translate-x-16 -translate-y-1/2 transform animate-pulse rounded-full bg-white/60"
					></div>
					<div
						class="absolute top-1/2 right-0 h-3 w-3 translate-x-16 -translate-y-1/2 transform animate-pulse rounded-full bg-white/60"
					></div>
				</div>
			{/if}
		</div>

		<!-- Success Messages -->
		<div class="space-y-4">
			{#if animationStep >= 1}
				<h1 class="animate-fade-in-up text-3xl font-bold text-white">Pembayaran Berhasil!</h1>
			{/if}

			{#if animationStep >= 2}
				<div class="animate-fade-in-up" style="animation-delay: 0.3s">
					<p class="mb-2 text-lg text-white/90">Transaksi Anda telah berhasil diproses</p>
					<div class="flex items-center justify-center space-x-2 text-emerald-200">
						<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1"
							/>
						</svg>
						<span class="font-medium">Order ID: #ORD-{Math.floor(Math.random() * 10000)}</span>
					</div>
				</div>
			{/if}

			{#if animationStep >= 3}
				<div class="animate-fade-in-up" style="animation-delay: 0.6s">
					<div class="mb-6 rounded-2xl border border-white/20 bg-white/10 p-4">
						<div class="mb-2 flex items-center justify-center space-x-2 text-white/80">
							<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
								/>
							</svg>
							<span class="text-sm">Terima kasih atas pembelian Anda!</span>
						</div>
						<p class="text-sm text-white/70">
							Pesanan Anda sedang diproses dan akan segera disiapkan.
						</p>
					</div>

					<button
						on:click={goToMenu}
						class="flex w-full transform items-center justify-center space-x-2 rounded-2xl bg-gradient-to-r from-emerald-500 to-green-600 px-8 py-4 font-semibold text-white transition-all duration-300 hover:scale-105 hover:from-emerald-600 hover:to-green-700 hover:shadow-lg"
					>
						<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M3 3h2l.4 2M7 13h10l4-8H5.4m0 0L7 13m0 0l-1.5 6M7 13l-1.5-6m0 0L4 5M7 13h10m0 0l1.5 6M17 13l1.5-6"
							/>
						</svg>
						<span>Kembali ke Menu</span>
					</button>
				</div>
			{/if}
		</div>
	</div>

	<!-- Additional 3D Elements -->
	{#if showAnimation}
		<div
			class="animate-float-1 absolute top-20 left-20 h-16 w-16 rounded-full bg-gradient-to-br from-yellow-400 to-orange-500 opacity-20"
		></div>
		<div
			class="animate-float-2 absolute right-20 bottom-20 h-12 w-12 rounded-full bg-gradient-to-br from-pink-400 to-purple-500 opacity-20"
		></div>
		<div
			class="animate-float-3 absolute top-1/2 left-10 h-8 w-8 rounded-full bg-gradient-to-br from-blue-400 to-indigo-500 opacity-20"
		></div>
		<div
			class="animate-float-1 absolute top-1/3 right-10 h-10 w-10 rounded-full bg-gradient-to-br from-green-400 to-emerald-500 opacity-20"
		></div>
	{/if}
</div>

<style>
	.preserve-3d {
		transform-style: preserve-3d;
	}

	.translateZ-12 {
		transform: translateZ(48px);
	}

	.translateZ-8 {
		transform: translateZ(32px);
	}

	.rotateX-45 {
		transform: rotateX(45deg) translateZ(32px);
	}

	.rotateY-45 {
		transform: rotateY(45deg) translateZ(32px);
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
			transform: rotateX(0deg) rotateY(360deg);
		}
	}

	@keyframes float {
		0%,
		100% {
			transform: translateY(0px) rotate(0deg);
		}
		50% {
			transform: translateY(-20px) rotate(180deg);
		}
	}

	@keyframes float-1 {
		0%,
		100% {
			transform: translateY(0px) rotate(0deg) scale(1);
		}
		50% {
			transform: translateY(-30px) rotate(180deg) scale(1.1);
		}
	}

	@keyframes float-2 {
		0%,
		100% {
			transform: translateY(0px) rotate(0deg) scale(1);
		}
		50% {
			transform: translateY(-25px) rotate(-180deg) scale(0.9);
		}
	}

	@keyframes float-3 {
		0%,
		100% {
			transform: translateY(0px) rotate(0deg) scale(1);
		}
		50% {
			transform: translateY(-35px) rotate(90deg) scale(1.2);
		}
	}

	@keyframes fade-in-up {
		0% {
			opacity: 0;
			transform: translateY(30px);
		}
		100% {
			opacity: 1;
			transform: translateY(0);
		}
	}

	@keyframes bounce-in {
		0% {
			transform: scale(0);
			opacity: 0;
		}
		50% {
			transform: scale(1.2);
			opacity: 0.8;
		}
		100% {
			transform: scale(1);
			opacity: 1;
		}
	}

	.animate-spin-3d {
		animation: spin-3d 4s linear infinite;
	}

	.animate-spin-slow {
		animation: spin 8s linear infinite;
	}

	.animate-float {
		animation: float 3s ease-in-out infinite;
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

	.animate-fade-in-up {
		animation: fade-in-up 0.8s ease-out forwards;
		opacity: 0;
	}

	.animate-bounce-in {
		animation: bounce-in 1s ease-out;
	}
</style>
