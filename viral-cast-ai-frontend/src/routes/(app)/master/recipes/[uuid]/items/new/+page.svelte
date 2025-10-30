<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/ui/Button.svelte';
	import type { PageData, ActionData } from './$types';

	export let data: PageData;
	export let form: ActionData;

	let isSubmitting = false;

	// Get ingredient name by UUID
	function getIngredientName(ingredientUuid: string): string {
		const ingredient = data.ingredients.find((i) => i.uuid === ingredientUuid);
		return ingredient?.name || 'Unknown Ingredient';
	}

	// Get ingredient UOM by UUID
	function getIngredientUOM(ingredientUuid: string): string {
		const ingredient = data.ingredients.find((i: any) => i.uuid === ingredientUuid);
		return ingredient?.base_uom?.code || '';
	}

	// Get UOM name by UUID
	function getUOMName(uomUuid: string): string {
		const uom = data.uoms.find((u: any) => u.uuid === uomUuid);
		return uom?.name || 'Unknown UOM';
	}

	// Get UOM code by UUID
	function getUOMCode(uomUuid: string): string {
		const uom = data.uoms.find((u: any) => u.uuid === uomUuid);
		return uom?.code || '';
	}

	let selectedIngredient = (form?.values as any)?.ingredient_uuid || '';
	let selectedUOM = (form?.values as any)?.uom_uuid || '';
	$: selectedIngredientUOM = selectedIngredient ? getIngredientUOM(selectedIngredient) : '';
	$: selectedUOMName = selectedUOM ? getUOMName(selectedUOM) : '';
	$: selectedUOMCode = selectedUOM ? getUOMCode(selectedUOM) : '';
</script>

<svelte:head>
	<title>Add Ingredient - {data.recipeSet?.name || 'Recipe'} - Viral Cast AI</title>
</svelte:head>

<main class="container mx-auto px-4 py-6">
	<!-- Header -->
	<div class="mb-6">
		<div class="mb-2 flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
			<a href="/master/recipes" class="hover:text-blue-600">Recipes</a>
			<span>/</span>
			<a href={data.recipeSet ? "/master/recipes/" + data.recipeSet.uuid : "/master/recipes"} class="hover:text-blue-600"
				>{data.recipeSet?.name || 'Recipe'}</a
			>
			<span>/</span>
			<span>Add Ingredient</span>
		</div>
		<h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Add Ingredient to Recipe</h1>
		<p class="mt-1 text-gray-600 dark:text-gray-300">
			Add a new ingredient to recipe "{data.recipeSet?.name}"
		</p>
	</div>

	<!-- Recipe Info -->
	{#if data.recipeSet}
		<div
			class="mb-6 rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-800 dark:bg-blue-900/30"
		>
			<div class="flex items-center">
				<div class="flex-shrink-0">
					<svg class="h-5 w-5 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
						<path
							fill-rule="evenodd"
							d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
							clip-rule="evenodd"
						/>
					</svg>
				</div>
				<div class="ml-3">
					<p class="text-sm text-blue-800 dark:text-blue-300">
						<span class="font-medium">Recipe:</span>
						{data.recipeSet.name}
					</p>
					<p class="mt-1 text-xs text-blue-600 dark:text-blue-300">
						Yield Quantity: {data.recipeSet.yield_qty} | Status: {data.recipeSet.is_active
							? 'Active'
							: 'Inactive'}
					</p>
				</div>
			</div>
		</div>
	{/if}

	<!-- Form -->
	<div class="rounded-lg border bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800">
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
				<!-- Ingredient Selection -->
				<div>
					<label
						for="ingredient_uuid"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
					>
						Ingredient <span class="text-red-500">*</span>
					</label>
					<select
						id="ingredient_uuid"
						name="ingredient_uuid"
						bind:value={selectedIngredient}
						class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {(
							form?.errors as any
						)?.ingredient_uuid
							? 'border-red-500'
							: ''}"
						required
					>
						<option value="">Select Ingredient</option>
						{#each data.ingredients as ingredient}
							<option value={ingredient.uuid}>{ingredient.name} ({ingredient.base_uom.code})</option
							>
						{/each}
					</select>
					{#if (form?.errors as any)?.ingredient_uuid}
						<p class="mt-1 text-sm text-red-600">{(form?.errors as any)?.ingredient_uuid}</p>
					{/if}
				</div>

				<!-- UOM Selection -->
				<div>
					<label
						for="uom_uuid"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
					>
						Unit of Measure <span class="text-red-500">*</span>
					</label>
					<select
						id="uom_uuid"
						name="uom_uuid"
						bind:value={selectedUOM}
						class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {(
							form?.errors as any
						)?.uom_uuid
							? 'border-red-500'
							: ''}"
						required
					>
						<option value="">Select Unit of Measure</option>
						{#each data.uoms as uom}
							<option value={uom.uuid}>{uom.name} ({uom.code})</option>
						{/each}
					</select>
					{#if (form?.errors as any)?.uom_uuid}
						<p class="mt-1 text-sm text-red-600">{(form?.errors as any)?.uom_uuid}</p>
					{/if}
				</div>

				<!-- Quantity -->
				<div>
					<label for="qty" class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
						Quantity <span class="text-red-500">*</span>
					</label>
					<div class="relative">
						<input
							type="number"
							id="qty"
							name="qty"
							value={(form?.values as any)?.qty || ''}
							placeholder="0"
							step="0.01"
							min="0.01"
							class="w-full rounded-md border border-gray-300 px-3 py-2 pr-16 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {(
								form?.errors as any
							)?.qty
								? 'border-red-500'
								: ''}"
							required
						/>
						{#if selectedUOMCode}
							<div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
								<span class="text-sm text-gray-500 dark:text-gray-400">{selectedUOMCode}</span>
							</div>
						{/if}
					</div>
					{#if (form?.errors as any)?.qty}
						<p class="mt-1 text-sm text-red-600">{(form?.errors as any)?.qty}</p>
					{/if}
					<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">Amount of ingredient required</p>
				</div>

				<!-- Waste Percentage -->
				<div>
					<label
						for="waste_pct"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
					>
						Waste Percentage
					</label>
					<div class="relative">
						<input
							type="number"
							id="waste_pct"
							name="waste_pct"
							value={(form?.values as any)?.waste_pct || '0'}
							placeholder="0"
							step="0.01"
							min="0"
							max="100"
							class="w-full rounded-md border border-gray-300 px-3 py-2 pr-8 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {(
								form?.errors as any
							)?.waste_pct
								? 'border-red-500'
								: ''}"
						/>
						<div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
							<span class="text-sm text-gray-500 dark:text-gray-400">%</span>
						</div>
					</div>
					{#if (form?.errors as any)?.waste_pct}
						<p class="mt-1 text-sm text-red-600">{(form?.errors as any)?.waste_pct}</p>
					{/if}
					<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
						Waste/loss percentage (0–100%)
					</p>
				</div>
			</div>

			<!-- Selected Ingredient Info -->
			{#if selectedIngredient || selectedUOM}
				<div class="mt-6 rounded-lg bg-gray-50 p-4 dark:bg-gray-900">
					<h3 class="mb-2 text-sm font-medium text-gray-900 dark:text-gray-100">
						Selection Information
					</h3>
					<div class="grid grid-cols-1 gap-4 text-sm md:grid-cols-2 lg:grid-cols-4">
						{#if selectedIngredient}
							{@const ingredient = data.ingredients.find((i: any) => i.uuid === selectedIngredient)}
							{#if ingredient}
								<div>
									<span class="text-gray-500 dark:text-gray-400">Ingredient:</span>
									<span class="ml-1 font-medium text-gray-900 dark:text-gray-100"
										>{ingredient.name}</span
									>
								</div>
								<div>
									<span class="text-gray-500 dark:text-gray-400">Base UOM:</span>
									<span class="ml-1 font-medium text-gray-900 dark:text-gray-100"
										>{ingredient.base_uom.name} ({ingredient.base_uom.code})</span
									>
								</div>
							{/if}
						{/if}
						{#if selectedUOM}
							<div>
								<span class="text-gray-500 dark:text-gray-400">Selected UOM:</span>
								<span class="ml-1 font-medium text-gray-900 dark:text-gray-100"
									>{selectedUOMName} ({selectedUOMCode})</span
								>
							</div>
						{/if}
						{#if selectedIngredient}
							{@const ingredient = data.ingredients.find((i: any) => i.uuid === selectedIngredient)}
							{#if ingredient}
								<div>
									<span class="text-gray-500 dark:text-gray-400">Stock:</span>
									<span class="ml-1 font-medium text-gray-900 dark:text-gray-100"
										>{ingredient.min_stock}</span
									>
								</div>
							{/if}
						{/if}
					</div>
				</div>
			{/if}

			<!-- General Error -->
			{#if (form?.errors as any)?.general}
				<div
					class="mt-4 rounded-md border border-red-200 bg-red-50 p-4 dark:border-red-800 dark:bg-red-900/30"
				>
					<p class="text-sm text-red-600">{(form?.errors as any)?.general}</p>
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
					{isSubmitting ? 'Saving...' : 'Add Item'}
				</button>
			</div>
		</form>
	</div>
</main>
