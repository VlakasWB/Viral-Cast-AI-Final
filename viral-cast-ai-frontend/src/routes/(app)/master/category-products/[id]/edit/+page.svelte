<script lang="ts">
	import ProductForm from '$lib/components/forms/ProductForm.svelte';
	import type { Product } from '$lib/types/product';
	let { data }: { data: { product: Product } } = $props();
</script>

<section class="max-w-2xl space-y-4">
	<h1 class="text-2xl font-semibold">Edit Product</h1>
	<ProductForm
		initial={data.product}
		submitLabel="Update"
		cancelHref="/master/products"
		includeUuidHidden={true}
	/>
</section>
