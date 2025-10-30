<script lang="ts">
	// Reusable product form: create/edit. Posts to parent route action.
	import type { Product } from '$lib/types/product.js';
	import Button from '$lib/components/ui/Button.svelte';
	import { buildImageUrl } from '$lib/utils/imageUrl.js';
    import { onMount } from 'svelte';
    import { getRecipeSetsApi } from '$lib/services/recipe.js';
    import type { RecipeSet } from '$lib/types/recipe.js';

	let {
		initial,
		submitLabel = 'Save',
		cancelHref = '/master/products',
		includeUuidHidden = false,
        includeRecipeSelect = false
	}: {
		initial?: Partial<Product>;
		submitLabel?: string;
		cancelHref?: string;
		includeUuidHidden?: boolean; // For edit page
        includeRecipeSelect?: boolean; // Show recipe selection on create
	} = $props();

	let name = $state(initial?.name ?? '');
	let category_uuid = $state(initial?.category_uuid ?? '');
	let sku = $state(initial?.sku ?? '');
	let price = $state(initial?.price ? parseFloat(initial.price.toString()) : 0);
	let status = $state(initial?.status ?? 'ACTIVE');
	let image_url = $state(initial?.image_url ?? '');
	let uuid = $state(initial?.uuid ?? '');

    // Recipe options state
    let recipeOptions = $state<RecipeSet[]>([]);
    let loadingRecipes = $state(false);
    let recipeError = $state<string | null>(null);

	// Image upload state
	let selectedFile: File | null = $state(null);
	let imagePreview = $state(buildImageUrl(initial?.image_url || null) ?? '');
	let isUploading = $state(false);

	// Available categories (in real app, this would be fetched from API)
	const availableCategories = [
		{ uuid: '19521769-90b2-4904-a1cf-e1bce2b1ea23', name: 'Beverages' },
		{ uuid: '2a8ceed7-1b26-48ac-a191-04d31f254419', name: 'Bakery' },
		{ uuid: '300b630f-2e59-48be-b3ad-2844c3286547', name: 'Snacks' },
		{ uuid: '690799f5-9eb6-49d9-9ca3-349a3428e191', name: 'Desserts' }
	];

	// Handle file selection
	function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];

		if (file) {
			selectedFile = file;

			// Create preview
			const reader = new FileReader();
			reader.onload = (e) => {
				imagePreview = e.target?.result as string;
			};
			reader.readAsDataURL(file);
		}
	}

	// Remove image
	function removeImage() {
		selectedFile = null;
		imagePreview = '';
		image_url = '';

		// Reset file input
		const fileInput = document.getElementById('product-image') as HTMLInputElement;
		if (fileInput) {
			fileInput.value = '';
		}
	}

    // Load recipes on mount if selection enabled
    onMount(async () => {
        if (!includeRecipeSelect) return;
        loadingRecipes = true;
        recipeError = null;
        try {
            const res = await getRecipeSetsApi({ page: 1, limit: 100, is_active: true });
            recipeOptions = res.data?.recipe_sets || [];
        } catch (err) {
            console.error('Failed to load recipes:', err);
            recipeError = 'Gagal memuat daftar resep';
        } finally {
            loadingRecipes = false;
        }
    });
</script>

<form method="POST" enctype="multipart/form-data" class="space-y-4">
	{#if includeUuidHidden}<input type="hidden" name="uuid" value={uuid} />{/if}

	<!-- Image Upload Section -->
	<div class="grid gap-2">
		<label for="product-image" class="text-sm opacity-80">Product Image</label>

		{#if imagePreview}
			<div class="relative inline-block">
				<img
					src={imagePreview}
					alt="Product preview"
					class="h-32 w-32 rounded-lg border border-gray-200 object-cover dark:border-gray-700"
				/>
				<button
					type="button"
					onclick={removeImage}
					class="absolute -top-2 -right-2 flex h-6 w-6 items-center justify-center rounded-full bg-red-500 text-xs text-white hover:bg-red-600"
				>
					×
				</button>
			</div>
		{/if}

		<input
			id="product-image"
			type="file"
			name="image"
			accept="image/*"
			onchange={handleFileSelect}
			class="form-input file:mr-4 file:rounded-full file:border-0 file:bg-blue-50 file:px-4 file:py-2 file:text-sm file:font-semibold file:text-blue-700 hover:file:bg-blue-100"
		/>
		<p class="text-xs text-gray-500">Upload an image for your product (optional)</p>
	</div>

	<div class="grid gap-1">
		<label for="product-name" class="text-sm opacity-80">Name</label>
		<input
			id="product-name"
			class="form-input"
			name="name"
			required
			bind:value={name}
			placeholder="e.g., Premium Coffee Blend"
		/>
	</div>

	<div class="grid gap-1">
		<label for="product-category" class="text-sm opacity-80">Category</label>
		<select
			id="product-category"
			class="form-input"
			name="category_uuid"
			required
			bind:value={category_uuid}
		>
			<option value="">Select Category</option>
			{#each availableCategories as category}
				<option value={category.uuid}>{category.name}</option>
			{/each}
		</select>
	</div>

	<div class="grid gap-1">
		<label for="product-sku" class="text-sm opacity-80">SKU</label>
		<input
			id="product-sku"
			class="form-input"
			name="sku"
			required
			bind:value={sku}
			placeholder="e.g., PCB-001"
		/>
	</div>

	<div class="grid gap-1">
		<label for="product-price" class="text-sm opacity-80">Price (IDR)</label>
		<input
			id="product-price"
			class="form-input"
			name="price"
			type="number"
			step="1000"
			min="0"
			required
			bind:value={price}
			placeholder="e.g., 85000"
		/>
	</div>

	<div class="grid gap-1">
		<label for="product-status" class="text-sm opacity-80">Status</label>
		<select id="product-status" class="form-input" name="status" required bind:value={status}>
			<option value="ACTIVE">Active</option>
			<option value="INACTIVE">Inactive</option>
		</select>
	</div>

    {#if includeRecipeSelect}
    <div class="grid gap-1">
        <label for="product-recipe" class="text-sm opacity-80">Recipe (optional)</label>
        <select id="product-recipe" class="form-input" name="current_recipe_uuid">
            <option value="">No recipe</option>
            {#if loadingRecipes}
                <option disabled>Loading recipes...</option>
            {:else}
                {#each recipeOptions as r}
                    <option value={r.uuid}>{r.name}</option>
                {/each}
            {/if}
        </select>
        {#if recipeError}
            <p class="text-xs text-red-600">{recipeError}</p>
        {/if}
        <p class="text-xs text-gray-500">Select a recipe to link to this product</p>
    </div>
    {/if}

	<!-- Hidden field for existing image URL -->
	{#if image_url && !selectedFile}
		<input type="hidden" name="existing_image_url" value={image_url} />
	{/if}

	<div class="flex gap-2 pt-2">
		<Button label="Cancel" color="deepYellow" href={cancelHref} />
		<Button label={submitLabel} color="sky" type="submit" disabled={isUploading} />
	</div>
</form>
