<script lang="ts">
	import IngredientForm from '$lib/components/forms/IngredientForm.svelte';
	import { t } from '$lib/stores/i18n';
</script>

<svelte:head>
	<title>{t('new_ingredient_stock_title')}</title>
</svelte:head>

<section class="space-y-4">
	<h1 class="text-xl font-semibold">{t('new_ingredient_stock_title')}</h1>
	<div class="max-w-md">
		<IngredientForm submitLabel={t('create')} />
	</div>
</section>
