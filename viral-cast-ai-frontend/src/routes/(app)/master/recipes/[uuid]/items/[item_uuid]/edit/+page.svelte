<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/ui/Button.svelte';
	import type { PageData, ActionData } from './$types';

	export let data: PageData;
	export let form: ActionData;

	let isSubmitting = false;
	let errs: Record<string, string> | undefined;

	const currentIngredient = (data.ingredients ?? []).find(
		(ingredient) => ingredient.uuid === data.recipeItem.ingredient_uuid
	);
	const currentIngredientBaseUom =
		currentIngredient?.base_uom?.code ?? currentIngredient?.base_uom?.name ?? '';
	const currentIngredientMinStock = currentIngredient?.min_stock ?? '';

	$: errs = form?.errors as Record<string, string> | undefined;
</script>

<svelte:head>
	<title>Edit Ingredient - {data.recipeSet.name} - Viral Cast AI</title>
	<meta name="description" content="Edit recipe ingredient quantity and waste percentage" />
</svelte:head>

<div class="container mx-auto px-4 py-6">
	<!-- Header -->
	<div class="mb-6">
		<div class="mb-2 flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
			<a href="/master/recipes" class="hover:text-blue-600 dark:hover:text-blue-400">Recipes</a>
			<span>/</span>
			<a href={"/master/recipes/" + data.recipeSet.uuid} class="hover:text-blue-600"
				>{data.recipeSet.name}</a
			>
			<span>/</span>
			<span>Edit Ingredient</span>
		</div>
		<h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Edit Recipe Ingredient</h1>
		<p class="mt-1 text-gray-600 dark:text-gray-400">
			Update ingredient quantity and waste percentage
		</p>
	</div>

	<!-- Recipe & Ingredient Info -->
	<div
		class="mb-6 rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-800 dark:bg-blue-900/20"
	>
		<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
			<div>
				<div class="mb-2 flex items-center">
					<svg
						class="mr-2 h-5 w-5 text-blue-400 dark:text-blue-300"
						fill="currentColor"
						viewBox="0 0 20 20"
					>
						<path
							fill-rule="evenodd"
							d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
							clip-rule="evenodd"
						/>
					</svg>
					<span class="text-sm font-medium text-blue-800 dark:text-blue-200"
						>Recipe Information</span
					>
				</div>
				<p class="text-sm text-blue-800 dark:text-blue-200">
					<span class="font-medium">Name:</span>
					{data.recipeSet.name}
				</p>
				<p class="mt-1 text-xs text-blue-600 dark:text-blue-300">
					Yield: {data.recipeSet.yield_qty} | Status: {data.recipeSet.is_active
						? 'Active'
						: 'Inactive'}
				</p>
			</div>

			{#if currentIngredient}
				<div>
					<div class="mb-2 flex items-center">
						<svg
							class="mr-2 h-5 w-5 text-blue-400 dark:text-blue-300"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
							/>
						</svg>
						<span class="text-sm font-medium text-blue-800 dark:text-blue-200"
							>Ingredient Information</span
						>
					</div>
					<p class="text-sm text-blue-800 dark:text-blue-200">
						<span class="font-medium">Name:</span>
						{currentIngredient.name}
					</p>
					<p class="mt-1 text-xs text-blue-600 dark:text-blue-300">
						UOM: {currentIngredientBaseUom} | Stock: {currentIngredientMinStock}
					</p>
				</div>
			{/if}
		</div>
		<div class="mt-3 border-t border-blue-200 pt-3 dark:border-blue-800">
			<p class="text-xs text-blue-600 dark:text-blue-300">
				<span class="font-medium">Note:</span> Ingredients cannot be changed after being added. To change
				the ingredient, delete this item and add a new ingredient.
			</p>
		</div>
	</div>

	<!-- Form -->
	<div class="rounded-lg border bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-900">
		<form
			method="POST"
			use:enhance={() => {
				isSubmitting = true;
				return async ({ update }) => {
					await update();
					isSubmitting = false;
				};
			}}
		>
			<div class="grid grid-cols-1 gap-6 md:grid-cols-2">
				<!-- Current Values Display -->
				<div class="rounded-lg bg-gray-50 p-4 md:col-span-2 dark:bg-gray-800">
					<h3 class="mb-3 text-sm font-medium text-gray-900 dark:text-gray-100">Current Values</h3>
					<div class="grid grid-cols-1 gap-4 text-sm md:grid-cols-2">
						<div>
							<span class="text-gray-500 dark:text-gray-400">Quantity:</span>
							<span class="ml-1 font-medium text-gray-900 dark:text-gray-100"
								>{data.recipeItem.qty} {currentIngredientBaseUom}</span
							>
						</div>
						<div>
							<span class="text-gray-500 dark:text-gray-400">Waste Percentage:</span>
							<span class="ml-1 font-medium text-gray-900 dark:text-gray-100"
								>{(data.recipeItem.waste_pct * 100).toFixed(2)}%</span
							>
						</div>
					</div>
				</div>

				<!-- Quantity -->
				<div>
					<label for="qty" class="mb-2 block text-sm font-medium text-gray-700">
						Quantity <span class="text-red-500">*</span>
					</label>
					<div class="relative">
						<input
							type="number"
							id="qty"
							name="qty"
							value={form?.values?.qty || data.recipeItem.qty}
							placeholder="0"
							step="0.01"
							min="0.01"
                            class="w-full rounded-md border border-gray-300 px-3 py-2 pr-16 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {errs
                                ?.qty
                                ? 'border-red-500 dark:border-red-600'
                                : ''}"
							required
						/>
						{#if currentIngredientBaseUom}
							<div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
								<span class="text-sm text-gray-500 dark:text-gray-400"
									>{currentIngredientBaseUom}</span
								>
							</div>
						{/if}
					</div>
                    {#if errs?.qty}
                        <p class="mt-1 text-sm text-red-600 dark:text-red-400">{errs.qty}</p>
                    {/if}
					<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">Amount of ingredient required</p>
				</div>

				<!-- Waste Percentage -->
				<div>
					<label for="waste_pct" class="mb-2 block text-sm font-medium text-gray-700">
						Waste Percentage
					</label>
					<div class="relative">
						<input
							type="number"
							id="waste_pct"
							name="waste_pct"
							value={form?.values?.waste_pct || data.recipeItem.waste_pct * 100}
							placeholder="0"
							step="0.01"
							min="0"
							max="100"
                            class="w-full rounded-md border border-gray-300 px-3 py-2 pr-8 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {errs
                                ?.waste_pct
                                ? 'border-red-500 dark:border-red-600'
                                : ''}"
						/>
						<div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
							<span class="text-sm text-gray-500 dark:text-gray-400">%</span>
						</div>
					</div>
                    {#if errs?.waste_pct}
                        <p class="mt-1 text-sm text-red-600 dark:text-red-400">{errs.waste_pct}</p>
                    {/if}
					<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
						Waste/loss percentage (0–100%)
					</p>
				</div>
			</div>

            <!-- General Error -->
            {#if errs?.general}
                <div
                    class="mt-4 rounded-md border border-red-200 bg-red-50 p-4 dark:border-red-800 dark:bg-red-900/20"
                >
                    <p class="text-sm text-red-600 dark:text-red-400">{errs.general}</p>
                </div>
            {/if}

			<!-- Form Actions -->
			<div class="mt-8 flex justify-end gap-4 border-t pt-6 dark:border-gray-700">
			<Button label="Cancel" color="deepYellow" href={data.recipeSet ? "/master/recipes/" + data.recipeSet.uuid : "/master/recipes"} />
				<button
					type="submit"
					disabled={isSubmitting}
					class="rounded-md bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
				>
					{isSubmitting ? 'Saving...' : 'Save Changes'}
				</button>
			</div>
		</form>
	</div>
</div>
