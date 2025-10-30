<script lang="ts">
	import ProductForm from '$lib/components/forms/ProductForm.svelte';
</script>

<svelte:head>
	<title>New Product</title>
</svelte:head>

<section class="space-y-4">
    <h1 class="text-xl font-semibold">New Product</h1>
    <div class="max-w-md">
        <ProductForm submitLabel="Create" includeRecipeSelect={true} />
    </div>
</section>
