<script lang="ts">
	import IngredientForm from '$lib/components/forms/IngredientForm.svelte';
	import { t } from '$lib/stores/i18n';
	import type { ActionData, PageData } from './$types';

	let {
		data,
		form
	}: {
		data: PageData;
		form: ActionData;
	} = $props();

	type IngredientFormValues = Partial<{
		name: string;
		description: string;
		base_uom: string;
		base_uom_uuid: string;
		min_stock: string | number;
		minimum_stock: string | number;
		shelf_life_days: string | number;
	}>;

	const formValues = $derived.by(
		() => form?.values as IngredientFormValues | undefined
	);
	const formErrors = $derived.by(
		() => form?.errors as Record<string, string | string[]> | undefined
	);
</script>

<svelte:head>
	<title>{t('new_ingredient_title')}</title>
</svelte:head>

<section class="space-y-4">
	<h1 class="text-xl font-semibold">{t('new_ingredient_title')}</h1>
	<div class="max-w-md">
		<IngredientForm
			submitLabel={t('create')}
			uoms={data.uoms}
			redirectTo={data.redirectTo}
			values={formValues}
			errors={formErrors}
		/>
	</div>
</section>
