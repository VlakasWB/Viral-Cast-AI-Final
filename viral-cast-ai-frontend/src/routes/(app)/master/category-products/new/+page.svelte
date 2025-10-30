<script lang="ts">
	import ProductForm from '$lib/components/forms/ProductForm.svelte';
</script>

<section class="max-w-2xl space-y-4">
	<h1 class="text-2xl font-semibold">New Product</h1>
	<ProductForm submitLabel="Create" cancelHref="/master/products" />
</section>
