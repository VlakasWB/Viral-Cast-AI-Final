<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import type { PageData } from './$types';
	import { deleteRecipeItemApi } from '$lib/services/recipe.js';
	import Button from '$lib/components/ui/Button.svelte';

	export let data: PageData;

	let showDeleteModal = false;
	let itemToDelete: any = null;
	let isDeleting = false;

	// Format date helper
	function formatDate(timestamp: number): string {
		return new Date(timestamp).toLocaleDateString('id-ID', {
			year: 'numeric',
			month: 'long',
			day: 'numeric'
		});
	}

	// Format datetime helper
	function formatDateTime(timestamp: number): string {
		return new Date(timestamp).toLocaleString('id-ID', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Delete recipe item
	function confirmDeleteItem(item: any) {
		itemToDelete = item;
		showDeleteModal = true;
	}

	async function deleteItem() {
		if (!itemToDelete) return;

		isDeleting = true;
		try {
			await deleteRecipeItemApi(itemToDelete.uuid);
			showDeleteModal = false;
			itemToDelete = null;
			// Refresh page
			goto($page.url.pathname, { replaceState: true });
		} catch (error) {
			console.error('Error deleting recipe item:', error);
			alert('Gagal menghapus item resep. Silakan coba lagi.');
		} finally {
			isDeleting = false;
		}
	}

	function cancelDelete() {
		showDeleteModal = false;
		itemToDelete = null;
	}

	// Calculate total waste percentage
	$: totalWastePercentage = data.recipeItems.reduce((sum, item) => sum + item.waste_pct * 100, 0);
</script>

<svelte:head>
	<title>Recipe Details: {data.recipeSet.name} - Viral Cast AI</title>
</svelte:head>

<main class="container mx-auto px-4 py-6">
	<!-- Breadcrumb -->
	<div class="mb-6 flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
		<a href="/master/recipes" class="hover:text-blue-600">Recipes</a>
		<span>/</span>
		<span>{data.recipeSet.name}</span>
	</div>

	<!-- Recipe Set Header -->
	<div class="mb-6 rounded-lg border bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800">
		<div class="mb-4 flex items-start justify-between">
			<div>
				<h1 class="mb-2 text-2xl font-bold text-gray-900 dark:text-gray-100">
					{data.recipeSet.name}
				</h1>
				<div class="flex items-center gap-4">
					<span
						class="inline-flex rounded-full px-3 py-1 text-sm font-semibold {data.recipeSet
							.is_active
							? 'bg-green-100 text-green-800'
							: 'bg-red-100 text-red-800'}"
					>
						{data.recipeSet.is_active ? 'Active' : 'Inactive'}
					</span>
					{#if data.product}
						<span class="text-sm text-gray-600 dark:text-gray-300"
							>Product: <span class="font-medium">{data.product.name}</span></span
						>
					{/if}
				</div>
			</div>
			<div class="flex gap-2">
				<a
					href={"/master/recipes/" + data.recipeSet.uuid + "/edit"}
					class="rounded-lg bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-700"
				>
					Edit Recipe
				</a>
				<a
					href={"/master/recipes/" + data.recipeSet.uuid + "/items/new"}
					class="rounded-lg bg-green-600 px-4 py-2 font-medium text-white transition-colors hover:bg-green-700"
				>
					+ Add Ingredient
				</a>
			</div>
		</div>

		<!-- Recipe Details Grid -->
		<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
			<div>
				<h3 class="mb-1 text-sm font-medium text-gray-500 dark:text-gray-400">Yield Quantity</h3>
				<p class="text-lg font-semibold text-gray-900 dark:text-gray-100">
					{data.recipeSet.yield_qty}
				</p>
			</div>
			<div>
				<h3 class="mb-1 text-sm font-medium text-gray-500 dark:text-gray-400">Effective Period</h3>
				<p class="text-sm text-gray-900 dark:text-gray-100">
					{formatDate(data.recipeSet.effective_from)} - {formatDate(data.recipeSet.effective_to)}
				</p>
			</div>
			<div>
				<h3 class="mb-1 text-sm font-medium text-gray-500 dark:text-gray-400">Total Ingredients</h3>
				<p class="text-lg font-semibold text-gray-900 dark:text-gray-100">
					{data.recipeItems.length} item
				</p>
			</div>
		</div>

		<!-- Timestamps -->
		<div class="mt-4 border-t border-gray-200 pt-4 dark:border-gray-700">
			<div class="grid grid-cols-1 gap-4 text-sm text-gray-600 md:grid-cols-2 dark:text-gray-300">
				<div>
					<span class="font-medium">Created:</span>
					{formatDateTime(data.recipeSet.created_at)}
				</div>
				<div>
					<span class="font-medium">Updated:</span>
					{formatDateTime(data.recipeSet.updated_at)}
				</div>
			</div>
		</div>
	</div>

	<!-- Recipe Items -->
	<div
		class="overflow-hidden rounded-lg border bg-white shadow-sm dark:border-gray-700 dark:bg-gray-800"
	>
		<div class="border-b border-gray-200 px-6 py-4 dark:border-gray-700">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
					Ingredient Composition
				</h2>
				{#if data.recipeItems.length > 0}
					<div class="text-sm text-gray-600 dark:text-gray-300">
						Total Waste: <span class="font-medium">{totalWastePercentage.toFixed(2)}%</span>
					</div>
				{/if}
			</div>
		</div>

		{#if data.recipeItems.length > 0}
			<div class="overflow-x-auto">
				<table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
					<thead class="bg-gray-50 dark:bg-gray-900">
						<tr>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-400"
							>
								Ingredient
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-400"
							>
								Quantity
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-400"
							>
								Waste %
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-400"
							>
								Added
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-400"
							>
								Actions
							</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-gray-200 bg-white dark:divide-gray-700 dark:bg-gray-800">
						{#each data.recipeItems as item}
							<tr class="hover:bg-gray-50 dark:hover:bg-gray-700">
								<td class="px-6 py-4 whitespace-nowrap">
									<div class="text-sm font-medium text-gray-900 dark:text-gray-100">
										{item.ingredient_name || `Ingredient ${item.ingredient_uuid.slice(0, 8)}...`}
									</div>
									{#if item.ingredient_base_uom}
										<div class="text-sm text-gray-500 dark:text-gray-400">
											UOM: {item.ingredient_base_uom}
										</div>
									{/if}
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<div class="text-sm text-gray-900 dark:text-gray-100">{item.qty}</div>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<div class="text-sm text-gray-900 dark:text-gray-100">
										{(item.waste_pct * 100).toFixed(2)}%
									</div>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<div class="text-sm text-gray-500 dark:text-gray-400">
										{formatDateTime(item.created_at)}
									</div>
								</td>
								<td class="space-x-2 px-6 py-4 text-sm font-medium whitespace-nowrap">
						<a
							href={"/master/recipes/" + data.recipeSet.uuid + "/items/" + item.uuid + "/edit"}
							class="text-indigo-600 hover:text-indigo-900"
						>
							Edit
						</a>
									<button
										on:click={() => confirmDeleteItem(item)}
										class="text-red-600 hover:text-red-900"
									>
										Delete
									</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{:else}
			<div class="px-6 py-12 text-center">
				<div class="mb-4 text-gray-400 dark:text-gray-500">
					<svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
						/>
					</svg>
				</div>
				<h3 class="mb-2 text-lg font-medium text-gray-900 dark:text-gray-100">
					No ingredients yet
				</h3>
				<p class="mb-4 text-gray-600 dark:text-gray-300">Add ingredients to complete this recipe</p>
				<a
					href={"/master/recipes/" + data.recipeSet.uuid + "/items/new"}
					class="inline-flex items-center rounded-md bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-700"
				>
					+ Add First Ingredient
				</a>
			</div>
		{/if}
	</div>
</main>

<!-- Delete Confirmation Modal -->
{#if showDeleteModal}
	<div class="bg-opacity-50 fixed inset-0 z-50 h-full w-full overflow-y-auto bg-gray-600">
		<div
			class="relative top-20 mx-auto w-96 rounded-md border bg-white p-5 shadow-lg dark:border-gray-700 dark:bg-gray-800"
		>
			<div class="mt-3 text-center">
				<h3 class="text-lg font-medium text-gray-900 dark:text-gray-100">Delete Confirmation</h3>
				<div class="mt-2 px-7 py-3">
					<p class="text-sm text-gray-500 dark:text-gray-300">
						Are you sure you want to remove this ingredient from the recipe? This action cannot be
						undone.
					</p>
				</div>
				<div class="mt-4 flex justify-center gap-4">
					<Button label="Cancel" color="deepYellow" onClick={cancelDelete} disabled={isDeleting} />
					<button
						on:click={deleteItem}
						disabled={isDeleting}
						class="rounded-md bg-red-600 px-4 py-2 text-base font-medium text-white shadow-sm hover:bg-red-700 focus:ring-2 focus:ring-red-500 focus:outline-none disabled:opacity-50"
					>
						{isDeleting ? 'Deleting...' : 'Delete'}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}
