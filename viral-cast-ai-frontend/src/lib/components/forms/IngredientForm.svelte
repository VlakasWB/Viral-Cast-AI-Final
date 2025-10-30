<script lang="ts">
	// Reusable ingredient form: create/edit. Posts to parent route action.
	import type { UOM } from '$lib/types/uom';
	import Button from '$lib/components/ui/Button.svelte';
	import { t } from '$lib/stores/i18n';
	import type { IngredientCatalogUom } from '$lib/types/ingredient-catalog';

	type IngredientFormInitial = Partial<{
		uuid: string;
		name: string;
		description: string | null;
		base_uom: string | IngredientCatalogUom | { uuid?: string; code?: string; name?: string };
		min_stock: string | number | null;
		minimum_stock: string | number | null;
		shelf_life_days: string | number | null;
	}>;

	type IngredientFormValues = Partial<{
		name: string;
		description: string | null;
		base_uom: string;
		base_uom_uuid: string;
		min_stock: string | number;
		minimum_stock: string | number;
		shelf_life_days: string | number;
	}>;

	const toInputString = (value: unknown): string => {
		if (value === null || value === undefined) return '';
		if (typeof value === 'number') {
			return Number.isFinite(value) ? String(value) : '';
		}
		return String(value ?? '');
	};

	const resolveBaseUomUuid = (
		source: IngredientFormInitial['base_uom'] | IngredientFormValues['base_uom'] | undefined
	): string => {
		if (!source) return '';
		if (typeof source === 'string') return source;
		if (typeof source === 'object') {
			return source.uuid ?? '';
		}
		return '';
	};

	let {
		initial,
		submitLabel = t('save'),
		cancelHref = '/master/ingredient-catalog',
		includeUuidHidden = false,
		uoms = [] as Array<Pick<UOM, 'uuid' | 'name' | 'code'>>,
		redirectTo,
		values,
		errors
	}: {
		initial?: IngredientFormInitial;
		submitLabel?: string;
		cancelHref?: string;
		includeUuidHidden?: boolean; // For edit page
		uoms?: Array<Pick<UOM, 'uuid' | 'name' | 'code'>>;
		redirectTo?: string;
		values?: IngredientFormValues;
		errors?: Record<string, string | string[]>;
	} = $props();

	const effectiveCancelHref = $derived.by(() => redirectTo ?? cancelHref);

	let name = $state(initial?.name ?? '');
	let description = $state(toInputString(initial?.description));
	let base_uom = $state(resolveBaseUomUuid(initial?.base_uom));
	let min_stock = $state(
		toInputString(
			initial?.min_stock !== undefined ? initial?.min_stock : initial?.minimum_stock
		)
	);
	let shelf_life_days = $state(toInputString(initial?.shelf_life_days));
	let uuid = $state(initial?.uuid ?? '');

	$effect(() => {
		if (!values) return;
		if (values.name !== undefined) {
			name = String(values.name ?? '');
		}
		if (values.description !== undefined) {
			description = String(values.description ?? '');
		}
		if (values.base_uom_uuid !== undefined) {
			base_uom = String(values.base_uom_uuid ?? '');
		} else if (values.base_uom !== undefined) {
			base_uom = resolveBaseUomUuid(values.base_uom);
		}
		if (values.min_stock !== undefined) {
			min_stock = toInputString(values.min_stock);
		} else if (values.minimum_stock !== undefined) {
			min_stock = toInputString(values.minimum_stock);
		}
		if (values.shelf_life_days !== undefined) {
			shelf_life_days = toInputString(values.shelf_life_days);
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

	const fallbackUOMs: Array<Pick<UOM, 'uuid' | 'name' | 'code'>> = [
		{ uuid: 'ab07b9dc-99e0-4591-bcb8-2ff8fa3fa034', name: 'Pieces', code: 'pcs' },
		{ uuid: '35e8af66-ae96-4313-9e3d-4014bef94d32', name: 'Dozen', code: 'dz' },
		{ uuid: 'd26ff637-0d8b-4e8f-860c-d78fa4a3c942', name: 'Milliliter', code: 'ml' },
		{ uuid: '2198a07b-f9a0-4924-99f6-85ef4fc78178', name: 'Liter', code: 'L' },
		{ uuid: 'b08b6dda-5c58-4d77-b0a7-d65e21177fb4', name: 'Gram', code: 'g' },
		{ uuid: '907e640d-e78e-4f1f-a603-4b2f67b0f268', name: 'Kilogram', code: 'kg' }
	];

	const availableUOMs = $derived.by(() => {
		const source = (uoms && uoms.length > 0 ? uoms : fallbackUOMs) ?? [];
		return source
			.map((uom) => {
				const labelParts = [uom.name?.trim() ?? ''];
				if (uom.code) labelParts.push(`(${uom.code})`);
				return {
					value: uom.uuid,
					label: labelParts.filter(Boolean).join(' ').trim()
				};
			})
			.filter((option) => option.value && option.label);
	});
</script>

<form method="POST" class="space-y-3">
	{#if includeUuidHidden}<input type="hidden" name="uuid" value={uuid} />{/if}
	<input type="hidden" name="redirectTo" value={effectiveCancelHref} />

	{#if generalError}
		<p class="text-sm text-red-600">{generalError}</p>
	{/if}

	<div class="grid gap-1">
		<input
			id="ingredient-name"
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

	<div class="grid gap-1">
		<label for="ingredient-base-uom" class="text-sm opacity-80">{t('form_label_base_uom')}</label>
		<select
			id="ingredient-base-uom"
			class="form-input"
			name="base_uom"
			required
			bind:value={base_uom}
		>
			<option value="">{t('select_uom')}</option>
			{#each availableUOMs as uom}
				<option value={uom.value}>{uom.label}</option>
			{/each}
		</select>
		{#if fieldErrors.base_uom}
			<p class="text-xs text-red-600">{toMessage(fieldErrors.base_uom as string | string[])}</p>
		{/if}
	</div>

	<div class="grid gap-1">
		<label for="ingredient-min-stock" class="text-sm opacity-80">{t('form_label_stock')}</label>
		<input
			id="ingredient-min-stock"
			class="form-input"
			name="min_stock"
			type="number"
			step="0.001"
			min="0"
			required
			bind:value={min_stock}
			placeholder={t('placeholder_stock')}
		/>
		{#if fieldErrors.min_stock}
			<p class="text-xs text-red-600">{toMessage(fieldErrors.min_stock as string | string[])}</p>
		{/if}
	</div>

	<div class="grid gap-1">
		<label for="ingredient-shelf-life" class="text-sm opacity-80">{t('form_label_shelf_life_days')}</label>
		<input
			id="ingredient-shelf-life"
			class="form-input"
			name="shelf_life_days"
			type="number"
			min="1"
			required
			bind:value={shelf_life_days}
			placeholder={t('placeholder_shelf_life_days')}
		/>
		{#if fieldErrors.shelf_life_days}
			<p class="text-xs text-red-600">
				{toMessage(fieldErrors.shelf_life_days as string | string[])}
			</p>
		{/if}
	</div>

	<div class="grid gap-1">
		<label for="ingredient-description" class="text-sm opacity-80">{t('form_label_description')}</label>
		<textarea
			id="ingredient-description"
			class="form-input"
			name="description"
			rows="3"
			bind:value={description}
			placeholder={t('placeholder_description')}
		></textarea>
		{#if fieldErrors.description}
			<p class="text-xs text-red-600">
				{toMessage(fieldErrors.description as string | string[])}
			</p>
		{/if}
	</div>

	<div class="flex gap-2 pt-2">
		<Button label={t('cancel')} color="deepYellow" href={effectiveCancelHref} />
		<Button label={submitLabel} color="sky" type="submit" />
	</div>
</form>
