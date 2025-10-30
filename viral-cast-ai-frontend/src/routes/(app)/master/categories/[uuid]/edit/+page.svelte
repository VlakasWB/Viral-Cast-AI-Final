<script lang="ts">
	import CategoryForm from '$lib/components/forms/CategoryForm.svelte';
	import type { Category } from '$lib/types/category.js';
	let { data }: { data: { category: Category } } = $props();
</script>

<section class="max-w-2xl space-y-4">
	<h1 class="text-2xl font-semibold">Edit Category</h1>
	<CategoryForm
		initial={data.category}
		submitLabel="Save Changes"
		cancelHref="/master/categories"
		includeUuidHidden={true}
	/>
</section>
