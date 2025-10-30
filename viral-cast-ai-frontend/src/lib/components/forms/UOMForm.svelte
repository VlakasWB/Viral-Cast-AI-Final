<script lang="ts">
	// Reusable UOM form: create/edit. Posts to parent route action.
	import type { UOM } from '$lib/types/uom.js';
	import Button from '$lib/components/ui/Button.svelte';
	import { t } from '$lib/stores/i18n';

	let {
		initial,
		submitLabel = t('save'),
		cancelHref = '/master/units-of-measurements',
		includeUuidHidden = false,
		redirectTo,
		values,
		errors
	}: {
		initial?: Partial<UOM> | undefined;
		submitLabel?: string;
		cancelHref?: string;
		includeUuidHidden?: boolean; // For edit page
		redirectTo?: string;
		values?: Partial<{ name: string; code: string }>;
		errors?: Record<string, string | string[]>;
	} = $props();

	const effectiveCancelHref = $derived.by(() => redirectTo ?? cancelHref);

	let name = $state(initial?.name ?? '');
	let code = $state(initial?.code ?? '');
	let uuid = $state(initial?.uuid ?? '');

	$effect(() => {
		if (!values) return;
		if (values.name !== undefined) {
			name = String(values.name ?? '');
		}
		if (values.code !== undefined) {
			code = String(values.code ?? '');
		}
	});

	const fieldErrors = $derived.by(() => errors ?? {});
	const toMessage = (err: string | string[] | undefined): string =>
		Array.isArray(err) ? err.filter(Boolean).join(', ') : err ?? '';
	const generalError = $derived.by(() =>
		toMessage(
			(fieldErrors as Record<string, string | string[] | undefined>).api ??
				(fieldErrors as Record<string, string | string[] | undefined>).general
		)
	);
</script>

<form method="POST" class="space-y-3">
	{#if includeUuidHidden}<input type="hidden" name="uuid" value={uuid} />{/if}
	<input type="hidden" name="redirectTo" value={effectiveCancelHref} />

	{#if generalError}
		<p class="text-sm text-red-600">{generalError}</p>
	{/if}

	<div class="grid gap-1">
		<label for="uom-code" class="text-sm opacity-80">{t('form_label_code')}</label>
		<input
			id="uom-code"
			class="form-input"
			name="code"
			required
			bind:value={code}
			placeholder={t('placeholder_code')}
		/>
		{#if fieldErrors.code}
			<p class="text-xs text-red-600">{toMessage(fieldErrors.code as string | string[])}</p>
		{/if}
	</div>

	<div class="grid gap-1">
		<label for="uom-name" class="text-sm opacity-80">{t('form_label_name')}</label>
		<input
			id="uom-name"
			class="form-input"
			name="name"
			required
			bind:value={name}
			placeholder={t('placeholder_name')}
		/>
		{#if fieldErrors.name}
			<p class="text-xs text-red-600">{toMessage(fieldErrors.name as string | string[])}</p>
		{/if}
	</div>

	<div class="flex gap-2 pt-2">
		<Button label={t('cancel')} color="deepYellow" href={effectiveCancelHref} />
		<Button label={submitLabel} color="orange" type="submit" />
	</div>
</form>
