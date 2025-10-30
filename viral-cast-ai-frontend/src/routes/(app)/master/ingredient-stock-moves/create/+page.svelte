<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { goto } from '$app/navigation';
	import { t } from '$lib/stores/i18n';
	import { getIngredientCatalogsApi } from '$lib/services/ingredient-catalog';

	let {
		data
	}: {
	data: {
			ingredients: Array<{ uuid: string; name: string }>;
		};
	} = $props();

	const initialIngredientOptions = data.ingredients.map((ingredient) => ({
		value: ingredient.uuid,
		label: ingredient.name
	}));

	const ingredientLabelCache = new Map(initialIngredientOptions.map((opt) => [opt.value, opt.label]));

	let ingredientOptions = $state(initialIngredientOptions);
	let ingredientUuid = $state('');
	let moveType = $state('');
	let quantity = $state('');
	let price = $state('');
	let isSubmitting = $state(false);
	let isIngredientLoading = $state(false);
	let errors = $state<Record<string, string>>({});
	let ingredientSearchVersion = 0;

	// ID: Opsi tipe pergerakan stok
	// EN: Options for stock move type
	const moveTypeOptions = $derived.by(() => [
		{ value: 'PRODUCTION', label: t('move_type_production') },
		{ value: 'ADJUSTMENT', label: t('move_type_adjustment') },
		{ value: 'WASTE', label: t('move_type_waste') },
		{ value: 'RETURN', label: t('move_type_return') }
	]);

	const numericInputClasses =
		'text-lg font-semibold tracking-tight text-slate-900 placeholder:text-slate-400 ' +
		'bg-slate-50/80 !border-0 rounded-2xl px-5 py-3 shadow-inner focus:ring-4 focus:ring-sky-200/70 ' +
		'focus:!border-sky-400 dark:bg-slate-900/50 dark:text-slate-100 dark:placeholder:text-slate-400 ' +
		'dark:focus:ring-sky-500/40';

	function validateForm() {
		const newErrors: Record<string, string> = {};

		if (!ingredientUuid) {
			newErrors.ingredientUuid = 'Ingredient harus dipilih';
		}

		if (!moveType) {
			newErrors.moveType = 'Tipe pergerakan harus dipilih';
		}

		if (!quantity || isNaN(parseFloat(quantity)) || parseFloat(quantity) <= 0) {
			newErrors.quantity = 'Quantity harus berupa angka positif';
		}

		if (!price || isNaN(parseFloat(price)) || parseFloat(price) <= 0) {
			newErrors.price = 'Harga harus berupa angka positif';
		}

		errors = newErrors;
		return Object.keys(newErrors).length === 0;
	}

	function handleCancel() {
		goto('/master/ingredient-stock-moves');
	}

	function applyIngredientOptions(list: Array<{ value: string; label: string }>) {
		for (const option of list) {
			if (option.value) {
				ingredientLabelCache.set(option.value, option.label);
			}
		}

		if (
			ingredientUuid &&
			ingredientUuid !== '' &&
			!list.some((option) => option.value === ingredientUuid)
		) {
			const cachedLabel = ingredientLabelCache.get(ingredientUuid);
			if (cachedLabel) {
				list = [...list, { value: ingredientUuid, label: cachedLabel }];
			}
		}

		ingredientOptions = list;
	}

async function handleIngredientSearch(query: string) {
		const trimmed = query.trim();
		ingredientSearchVersion += 1;
		const currentVersion = ingredientSearchVersion;

		if (!trimmed) {
			applyIngredientOptions([...initialIngredientOptions]);
			return;
		}

		isIngredientLoading = true;
		try {
			const response = await getIngredientCatalogsApi(1, 20, trimmed);
			if (ingredientSearchVersion !== currentVersion) return;
			const mapped = response.items.map((ingredient) => ({
				value: ingredient.uuid,
				label: ingredient.name
			}));
			applyIngredientOptions(mapped);
		} catch (error) {
			console.error('Failed to search ingredients:', error);
		} finally {
			if (ingredientSearchVersion === currentVersion) {
				isIngredientLoading = false;
			}
	}
}

applyIngredientOptions([...initialIngredientOptions]);
</script>

<section class="mx-auto max-w-5xl p-4">
	<div class="rounded-3xl border border-gray-200/60 bg-white/95 p-6 text-gray-900 shadow-xl dark:border-gray-700/60 dark:bg-gray-900 dark:text-gray-100">
		<h1 class="text-2xl font-semibold">{t('new_ingredient_stock_move_title')}</h1>
		<p class="mb-6 text-sm text-gray-600 dark:text-gray-400">{t('ingredient_stock_moves_subtitle')}</p>

		<form method="POST" use:enhance={() => {
			isSubmitting = true;
			
			return async ({ result }) => {
				isSubmitting = false;
				
				if (result.type === 'success') {
					goto('/master/ingredient-stock-moves');
				}
			};
		}}>
		<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				<div class="space-y-3">
					<Select
						label={t('nav_ingredients')}
						name="ingredientUuid"
						bind:value={ingredientUuid}
						options={ingredientOptions}
						required
						error={errors.ingredientUuid}
						placeholder={t('select_ingredient')}
						searchable
						searchPlaceholder={t('stock_move_search_placeholder')}
						loading={isIngredientLoading}
						onSearch={handleIngredientSearch}
					/>
				</div>

				<div class="space-y-3">
					<Select
							label={t('move_type')}
							name="moveType"
							bind:value={moveType}
							options={moveTypeOptions}
							required
							error={errors.moveType}
							placeholder={t('select_move_type')}
						/>
				</div>

				<div class="space-y-3">
					<Input
						label={`📦 ${t('quantity')}`}
						name="quantity"
						type="number"
						step="0.001"
						min="0"
						bind:value={quantity}
						placeholder="0.000"
						required
						error={errors.quantity}
						classes={numericInputClasses}
					/>
					<p class="text-xs font-medium uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
						{t('quantity')} ({t('nav_ingredients')})
					</p>
				</div>

				<div class="space-y-3">
					<Input
						label={`💸 ${t('col_price')}`}
						name="price"
						type="number"
						step="0.01"
						min="0"
						bind:value={price}
						placeholder="0"
						required
						error={errors.price}
						classes={numericInputClasses}
					/>
					<p class="text-xs font-medium uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
						Rp
					</p>
				</div>

			</div>

			<div class="flex justify-end space-x-4 mt-8">
				<Button
					type="button"
					color="deepYellow"
					label={t('cancel')}
					onClick={handleCancel}
				/>
				<Button
					type="submit"
					color="ocean"
					label={isSubmitting ? t('saving') : t('save')}
					disabled={isSubmitting}
				/>
			</div>
		</form>
	</div>
</section>
