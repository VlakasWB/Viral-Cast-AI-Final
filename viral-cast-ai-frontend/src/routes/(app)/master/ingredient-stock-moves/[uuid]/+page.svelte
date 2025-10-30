<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import { goto } from '$app/navigation';
	import type { IngredientStockMove } from '$lib/types/ingredient-stock-moves';
	import { t, locale } from '$lib/stores/i18n';
	import { get } from 'svelte/store';

	let {
		data
	}: {
		data: {
			stockMove: IngredientStockMove;
		};
	} = $props();

	// ID: Fungsi untuk memformat tanggal
	// EN: Function to format date
	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleDateString(get(locale) === 'id' ? 'id-ID' : 'en-US', {
			year: 'numeric',
			month: 'long',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// ID: Fungsi untuk memformat tipe pergerakan
	// EN: Function to format move type
	function formatMoveType(moveType: string): string {
		const types: Record<string, string> = {
			PRODUCTION: t('move_type_production'),
			ADJUSTMENT: t('move_type_adjustment'),
			WASTE: t('move_type_waste'),
			RETURN: t('move_type_return')
		};
		return types[moveType] || moveType;
	}

	// ID: Fungsi untuk mendapatkan class badge berdasarkan tipe pergerakan
	// EN: Function to get badge class based on move type
	function getMoveTypeBadgeClass(moveType: string): string {
		const classes: Record<string, string> = {
			PRODUCTION: 'bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-300',
			ADJUSTMENT: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
			WASTE: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300',
			RETURN: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300'
		};
		return classes[moveType] || 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300';
	}

	function handleBack() {
		goto('/master/ingredient-stock-moves');
	}

	function handleEdit() {
		goto(`/master/ingredient-stock-moves/${data.stockMove.uuid}/edit`);
	}

	function resolveUnitLabel(...values: Array<string | null | undefined>) {
		for (const value of values) {
			if (value === null || value === undefined) continue;
			const text = String(value).trim();
			if (text) {
				return text;
			}
		}
		return '';
	}

	function formatQuantityDisplay(
		value: number | string | null | undefined,
		...unitCandidates: Array<string | null | undefined>
	) {
		if (value === null || value === undefined) return '-';
		const quantityText =
			typeof value === 'number' && Number.isFinite(value)
				? String(value)
				: String(value ?? '').trim() || '0';
		const unitLabel = resolveUnitLabel(...unitCandidates);
		return unitLabel ? `${quantityText} ${unitLabel}` : quantityText;
	}
</script>

<svelte:head>
	<title>{t('detail_ingredient_stock_move_title')}</title>
</svelte:head>

<div class="container mx-auto p-4 text-gray-900 dark:text-gray-100 transition-colors">
	<div
		class="rounded-3xl border border-gray-200/70 bg-white/95 p-6 shadow-xl backdrop-blur-sm dark:border-gray-700/60 dark:bg-slate-900/80 dark:shadow-[0_20px_45px_rgba(8,15,40,0.45)]"
	>
		<!-- Header -->
		<div class="flex items-center justify-between mb-6">
			<h1 class="text-2xl font-bold text-slate-900 dark:text-slate-100">
				{t('detail_ingredient_stock_move_title')}
			</h1>
			<div class="flex gap-2">
				<Button variant="outline" color="gray" label={t('back')} onClick={handleBack} />
				<Button color="violet" label={t('edit')} onClick={handleEdit} />
			</div>
		</div>

		<!-- Detail Information -->
		<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
			<div class="space-y-4">
				<div>
					<div class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						{t('store')}
					</div>
					<p class="text-gray-900 dark:text-gray-100 font-medium">
						{data.stockMove.store?.name ?? t('not_available')}
					</p>
				</div>

				<div>
					<div class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						{t('nav_ingredients')}
					</div>
					<p class="text-gray-900 dark:text-gray-100 font-medium">
						{data.stockMove.ingredient?.name ?? t('not_available')}
					</p>
				</div>

				<div>
					<div class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						{t('move_type')}
					</div>
					<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getMoveTypeBadgeClass(data.stockMove.move_type)}">
						{formatMoveType(data.stockMove.move_type)}
					</span>
				</div>

				<div>
					<div class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						{t('quantity')}
					</div>
					<p class="text-gray-900 dark:text-gray-100 font-medium">
						{formatQuantityDisplay(
							data.stockMove.quantity,
							data.stockMove.unit_of_measure_code,
							data.stockMove.unit_of_measure?.code,
							data.stockMove.unit_of_measure?.symbol,
							data.stockMove.unit_of_measure_name,
							data.stockMove.unit_of_measure?.name,
							data.stockMove.ingredient?.base_uom
						)}
					</p>
				</div>
			</div>

			<div class="space-y-4">
				<div>
					<div class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						{t('col_created_at')}
					</div>
					<p class="text-gray-900 dark:text-gray-100">
						{formatDate(data.stockMove.created_at)}
					</p>
				</div>

				<div>
					<div class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						{t('col_updated_at')}
					</div>
					<p class="text-gray-900 dark:text-gray-100">
						{formatDate(data.stockMove.updated_at)}
					</p>
				</div>
			</div>
		</div>
	</div>
</div>
