<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/ui/Button.svelte';
	import type { ActionData, PageData } from './$types';

	export let data: PageData;
	export let form: ActionData;

	let isSubmitting = false;
	let hasEffectivePeriod = Boolean(data.recipeSet.effective_from && data.recipeSet.effective_to);
	let errs: Record<string, string> | undefined;

	// Normalize error map to avoid union type issues
	$: errs = form?.errors as Record<string, string> | undefined;

	// Convert timestamps to date strings for input fields
	function timestampToDateString(timestamp: number): string {
		return new Date(timestamp).toISOString().split('T')[0];
	}

	// Get product name by UUID
	function getProductName(productUuid: string): string {
		const product = data.products.find(
			(p: { uuid: string; name: string }) => p.uuid === productUuid
		);
		return product?.name || 'Unknown Product';
	}
</script>

<svelte:head>
	<title>Edit Recipe: {data.recipeSet.name} - Viral Cast AI</title>
	<meta
		name="description"
		content="Update recipe information including name, yield, status, and effective period"
	/>
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
			<span>Edit</span>
		</div>
		<h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Edit Recipe</h1>
		<p class="mt-1 text-gray-600 dark:text-gray-400">Update recipe information</p>
	</div>

	<!-- Current Product Info -->
	<div
		class="mb-6 rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-800 dark:bg-blue-900/20"
	>
		<div class="flex items-center">
			<div class="flex-shrink-0">
				<svg
					class="h-5 w-5 text-blue-400 dark:text-blue-300"
					fill="currentColor"
					viewBox="0 0 20 20"
				>
					<path
						fill-rule="evenodd"
						d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
						clip-rule="evenodd"
					/>
				</svg>
			</div>
			<div class="ml-3">
				<p class="text-sm text-blue-800 dark:text-blue-200">
					<span class="font-medium">Product:</span>
					{getProductName(data.recipeSet.product_uuid)}
				</p>
				<p class="mt-1 text-xs text-blue-600 dark:text-blue-300">
					The product cannot be changed after the recipe is created. To change the product, create a
					new recipe.
				</p>
			</div>
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
				<!-- Recipe Name -->
				<div class="md:col-span-2">
					<label for="name" class="mb-2 block text-sm font-medium text-gray-700">
						Recipe Name <span class="text-red-500">*</span>
					</label>
					<input
						type="text"
						id="name"
						name="name"
						value={form?.values?.name || data.recipeSet.name}
						placeholder="Enter recipe name"
                        class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {errs
                            ?.name
                            ? 'border-red-500 dark:border-red-600'
                            : ''}"
						required
					/>
                    {#if errs?.name}
                        <p class="mt-1 text-sm text-red-600 dark:text-red-400">{errs.name}</p>
                    {/if}
				</div>

				<!-- Yield Quantity -->
				<div>
					<label for="yield_qty" class="mb-2 block text-sm font-medium text-gray-700">
						Yield Quantity <span class="text-red-500">*</span>
					</label>
					<input
						type="number"
						id="yield_qty"
						name="yield_qty"
						value={form?.values?.yield_qty || data.recipeSet.yield_qty}
						placeholder="0"
						step="0.01"
						min="0.01"
                        class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {errs
                            ?.yield_qty
                            ? 'border-red-500 dark:border-red-600'
                            : ''}"
                        required
                    />
                    {#if errs?.yield_qty}
                        <p class="mt-1 text-sm text-red-600 dark:text-red-400">{errs.yield_qty}</p>
                    {/if}
					<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
						The amount of production yield from this recipe
					</p>
				</div>

				<!-- Active Status -->
				<div class="flex items-center">
					<div class="flex h-5 items-center">
						<input
							id="is_active"
							name="is_active"
							type="checkbox"
							checked={form?.values?.is_active !== undefined
								? form.values.is_active
								: data.recipeSet.is_active}
							class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-gray-700"
						/>
					</div>
					<div class="ml-3 text-sm">
						<label for="is_active" class="font-medium text-gray-700 dark:text-gray-300"
							>Active</label
						>
						<p class="text-gray-500 dark:text-gray-400">
							This recipe will be active and available for use
						</p>
					</div>
				</div>

				<!-- Effective From -->
				<div>
					<label
						for="effective_from"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
					>
						Effective From {#if hasEffectivePeriod}<span class="text-red-500">*</span>{/if}
					</label>
					<input
						type="date"
						id="effective_from"
						name="effective_from"
						value={form?.values?.effective_from ||
							(hasEffectivePeriod && data.recipeSet.effective_from
								? timestampToDateString(data.recipeSet.effective_from)
								: '')}
						disabled={!hasEffectivePeriod}
                        class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {errs
                            ?.effective_from
                            ? 'border-red-500 dark:border-red-600'
                            : ''} {!hasEffectivePeriod ? 'cursor-not-allowed bg-gray-100 dark:bg-gray-800' : ''}"
                        required={hasEffectivePeriod}
                    />
                    {#if errs?.effective_from}
                        <p class="mt-1 text-sm text-red-600 dark:text-red-400">{errs.effective_from}</p>
                    {/if}
				</div>

				<!-- Has Effective Period -->
				<div class="flex items-center">
					<div class="flex h-5 items-center">
						<input
							id="has_effective_period"
							type="checkbox"
							bind:checked={hasEffectivePeriod}
							class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
						/>
					</div>
					<div class="ml-3 text-sm">
						<label for="has_effective_period" class="font-medium text-gray-700 dark:text-gray-300"
							>Set Effective Period</label
						>
						<p class="text-gray-500 dark:text-gray-400">
							Enable to set specific effective dates for this recipe
						</p>
					</div>
				</div>

				<!-- Effective To -->
				<div>
					<label
						for="effective_to"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
					>
						Effective To {#if hasEffectivePeriod}<span class="text-red-500">*</span>{/if}
					</label>
					<input
						type="date"
						id="effective_to"
						name="effective_to"
						value={form?.values?.effective_to ||
							(hasEffectivePeriod && data.recipeSet.effective_to
								? timestampToDateString(data.recipeSet.effective_to)
								: '')}
						disabled={!hasEffectivePeriod}
                        class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {errs
                            ?.effective_to
                            ? 'border-red-500 dark:border-red-600'
                            : ''} {!hasEffectivePeriod ? 'cursor-not-allowed bg-gray-100 dark:bg-gray-800' : ''}"
                        required={hasEffectivePeriod}
                    />
                    {#if errs?.effective_to}
                        <p class="mt-1 text-sm text-red-600 dark:text-red-400">{errs.effective_to}</p>
                    {/if}
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
					{isSubmitting ? 'Saving...' : 'Update Recipe'}
				</button>
			</div>
		</form>
	</div>
</div>
