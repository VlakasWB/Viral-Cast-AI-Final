<script lang="ts">
	import UOMForm from '$lib/components/forms/UOMForm.svelte';
	import { t } from '$lib/stores/i18n';
	import type { ActionData, PageData } from './$types';

	let {
		data,
		form
	}: {
		data: PageData;
		form: ActionData;
	} = $props();

	type UOMFormValues = Partial<{ name: string; code: string }>;
	const formValues = $derived.by(
		() => form?.values as UOMFormValues | undefined
	);
	const formErrors = $derived.by(
		() => form?.errors as Record<string, string | string[]> | undefined
	);
</script>

<svelte:head>
	<title>{t('new_uom_title')}</title>
</svelte:head>

<section class="space-y-4">
	<h1 class="text-xl font-semibold">{t('new_uom_title')}</h1>
	<div class="max-w-md">
		<UOMForm
			submitLabel={t('create')}
			redirectTo={data.redirectTo}
			values={formValues}
			errors={formErrors}
		/>
	</div>
</section>
