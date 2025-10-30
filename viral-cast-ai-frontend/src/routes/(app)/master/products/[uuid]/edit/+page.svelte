<script lang="ts">
	import ProductForm from '$lib/components/forms/ProductForm.svelte';
	import type { Product } from '$lib/types/product.js';

	let { data }: { data: { product: Product } } = $props();
</script>

<svelte:head>
	<title>Edit Product</title>
</svelte:head>

<section class="space-y-4">
	<h1 class="text-xl font-semibold">Edit Product</h1>
	<div class="max-w-md">
		<ProductForm initial={data.product} submitLabel="Update" includeUuidHidden={true} />
	</div>
</section>
