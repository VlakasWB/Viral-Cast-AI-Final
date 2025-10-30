<script lang="ts">
	import IngredientForm from '$lib/components/forms/IngredientForm.svelte';
</script>

<svelte:head>
	<title>New Ingredient</title>
</svelte:head>

<section class="space-y-4">
	<h1 class="text-xl font-semibold">New Ingredient</h1>
	<div class="max-w-md">
		<IngredientForm submitLabel="Create" />
	</div>
</section>
