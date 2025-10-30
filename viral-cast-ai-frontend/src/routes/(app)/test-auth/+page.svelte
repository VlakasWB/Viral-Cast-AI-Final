<script lang="ts">
	let { data } = $props();
</script>

<svelte:head>
	<title>Test Authentication - Cashier App</title>
</svelte:head>

<div class="mx-auto mt-20 max-w-md">
	<div
		class="rounded-xl border border-gray-200 bg-white p-8 shadow-lg dark:border-gray-700 dark:bg-gray-800"
	>
		<h1 class="mb-6 text-center text-2xl font-bold text-gray-900 dark:text-white">
			🔐 Cashier Login
		</h1>

		<div class="space-y-4">
			<form method="POST" action="?/login">
				<button
					type="submit"
					class="w-full rounded-lg bg-blue-600 px-6 py-3 font-medium text-white transition-colors hover:bg-blue-700"
				>
					Login as Cashier
				</button>
			</form>

			<form method="POST" action="?/logout">
				<button
					type="submit"
					class="w-full rounded-lg bg-red-600 px-6 py-3 font-medium text-white transition-colors hover:bg-red-700"
				>
					Logout
				</button>
			</form>

			<div class="text-center">
				<a
					href="/products"
					class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
				>
					Go to Shop
				</a>
			</div>
		</div>

		<div class="mt-8 rounded-lg bg-gray-50 p-4 dark:bg-gray-700">
			<h3 class="mb-2 font-medium text-gray-900 dark:text-white">Instructions:</h3>
			<ol class="space-y-1 text-sm text-gray-600 dark:text-gray-400">
				<li>1. Click "Login as Cashier" to authenticate</li>
				<li>2. Go to Shop and add items to cart</li>
				<li>3. Proceed to checkout and process payment</li>
				<li>4. Check if order creation works</li>
			</ol>
		</div>
	</div>
</div>
