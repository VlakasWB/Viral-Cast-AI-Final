<script lang="ts">
	import CategoryForm from '$lib/components/forms/CategoryForm.svelte';
</script>

<section class="max-w-2xl space-y-4">
	<h1 class="text-2xl font-semibold">New Category</h1>
	<CategoryForm submitLabel="Create" cancelHref="/master/categories" />
</section>
