<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageData, ActionData } from './$types';
	import Button from '$lib/components/ui/Button.svelte';

	export let form: ActionData;

	let isSubmitting = false;
	let hasEffectivePeriod = false;

	// Get today's date for default values
	const today = new Date().toISOString().split('T')[0];
	const nextYear = new Date();
	nextYear.setFullYear(nextYear.getFullYear() + 1);
	const defaultEndDate = nextYear.toISOString().split('T')[0];

	// Normalize error map to avoid union type issues
	$: errs = (form?.errors as Record<string, string> | undefined);
	$: generalError = errs?.general;
</script>

<svelte:head>
	<title>Add New Recipe - Viral Cast AI</title>
</svelte:head>

<main class="container mx-auto px-4 py-6">
	<!-- Header -->
	<div class="mb-6">
		<div class="mb-2 flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
			<a href="/master/recipes" class="hover:text-blue-600">Recipes</a>
			<span>/</span>
			<span>Add New</span>
		</div>
		<h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Add New Recipe</h1>
		<p class="mt-1 text-gray-600 dark:text-gray-300">Create a new recipe</p>
	</div>

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
				<!-- Recipe Name -->
				<div class="md:col-span-2">
					<label for="name" class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-200">
						Recipe Name <span class="text-red-500">*</span>
					</label>
					<input
						type="text"
						id="name"
						name="name"
						value={form?.values?.name || ''}
						placeholder="Enter recipe name"
					class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {errs
						?.name
						? 'border-red-500'
						: ''}"
						required
					/>
				{#if errs?.name}
					<p class="mt-1 text-sm text-red-600">{errs.name}</p>
				{/if}
				</div>

				<!-- Yield Quantity -->
				<div>
					<label
						for="yield_qty"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-200"
					>
						Yield Quantity <span class="text-red-500">*</span>
					</label>
					<input
						type="number"
						id="yield_qty"
						name="yield_qty"
						value={form?.values?.yield_qty || ''}
						placeholder="0"
						step="0.01"
						min="0.01"
					class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {errs
						?.yield_qty
						? 'border-red-500'
						: ''}"
						required
					/>
				{#if errs?.yield_qty}
					<p class="mt-1 text-sm text-red-600">{errs.yield_qty}</p>
				{/if}
					<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
						Total yield produced from this recipe
					</p>
				</div>

				<!-- Active Status -->
				<div class="flex items-center">
					<div class="flex h-5 items-center">
						<input
							id="is_active"
							name="is_active"
							type="checkbox"
							checked={form?.values?.is_active !== undefined ? form.values.is_active : true}
							class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
						/>
					</div>
					<div class="ml-3 text-sm">
						<label for="is_active" class="font-medium text-gray-700 dark:text-gray-200"
							>Active</label
						>
						<p class="text-gray-500 dark:text-gray-400">This recipe will be active and usable</p>
					</div>
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
						<label for="has_effective_period" class="font-medium text-gray-700 dark:text-gray-200"
							>Set Effective Period</label
						>
						<p class="text-gray-500 dark:text-gray-400">
							Enable to set specific effective dates for this recipe
						</p>
					</div>
				</div>

				<!-- Effective From -->
				<div>
					<label
						for="effective_from"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-200"
					>
						Effective From {#if hasEffectivePeriod}<span class="text-red-500">*</span>{/if}
					</label>
					<input
						type="date"
						id="effective_from"
						name="effective_from"
						value={form?.values?.effective_from || (hasEffectivePeriod ? today : '')}
						disabled={!hasEffectivePeriod}
					class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {errs
						?.effective_from
						? 'border-red-500'
						: ''} {!hasEffectivePeriod ? 'cursor-not-allowed bg-gray-100 dark:bg-gray-800' : ''}"
						required={hasEffectivePeriod}
					/>
				{#if errs?.effective_from}
					<p class="mt-1 text-sm text-red-600">{errs.effective_from}</p>
				{/if}
				</div>

				<!-- Effective To -->
				<div>
					<label
						for="effective_to"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-200"
					>
						Effective To {#if hasEffectivePeriod}<span class="text-red-500">*</span>{/if}
					</label>
					<input
						type="date"
						id="effective_to"
						name="effective_to"
						value={form?.values?.effective_to || (hasEffectivePeriod ? defaultEndDate : '')}
						disabled={!hasEffectivePeriod}
					class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 {errs
						?.effective_to
						? 'border-red-500'
						: ''} {!hasEffectivePeriod ? 'cursor-not-allowed bg-gray-100 dark:bg-gray-800' : ''}"
						required={hasEffectivePeriod}
					/>
				{#if errs?.effective_to}
					<p class="mt-1 text-sm text-red-600">{errs.effective_to}</p>
				{/if}
				</div>
			</div>

			{#if generalError}
				<div
					class="mt-4 rounded-md border border-red-200 bg-red-50 p-4 dark:border-red-700 dark:bg-red-900/20"
				>
					<p class="text-sm text-red-600 dark:text-red-400">{generalError}</p>
				</div>
			{/if}

			<!-- Form Actions -->
			<div class="mt-8 flex justify-end gap-4 border-t pt-6 dark:border-gray-700">
				<Button label="Cancel" color="deepYellow" href="/master/recipes" />
				<button
					type="submit"
					disabled={isSubmitting}
					class="rounded-md bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
				>
					{isSubmitting ? 'Saving...' : 'Save Recipe'}
				</button>
			</div>
		</form>
	</div>
</main>
