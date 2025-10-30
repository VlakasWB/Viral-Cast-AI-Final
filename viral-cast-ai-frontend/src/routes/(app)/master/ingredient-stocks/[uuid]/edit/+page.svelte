<script lang="ts">
	import IngredientForm from '$lib/components/forms/IngredientForm.svelte';
	import type { Ingredient } from '$lib/types/ingredient.js';
	import { t } from '$lib/stores/i18n';

	let { data }: { data: { ingredient: Ingredient } } = $props();
</script>

<svelte:head>
	<title>{t('edit_ingredient_stock_title')}</title>
</svelte:head>

<section class="space-y-4">
	<h1 class="text-xl font-semibold">{t('edit_ingredient_stock_title')}</h1>
	<div class="max-w-md">
		<IngredientForm initial={data.ingredient} submitLabel={t('update')} includeUuidHidden={true} />
	</div>
</section>
