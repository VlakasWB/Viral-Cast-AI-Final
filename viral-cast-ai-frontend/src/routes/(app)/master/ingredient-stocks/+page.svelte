<script lang="ts">
import SearchBox from '$lib/components/SearchBox.svelte';
import Button from '$lib/components/ui/Button.svelte';
import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
import TablePagination from '$lib/components/TablePagination.svelte';
	import type { IngredientStock } from '$lib/types/ingredientStock';
	import type { StoreIngredientPredictionsData } from '$lib/types/store-ingredient-prediction';
	import { goto, invalidateAll } from '$app/navigation';
	import { formatCurrency } from '$lib/utils/format';
	import { t } from '$lib/stores/i18n';

	let {
		data
	}: {
		data: {
			items: IngredientStock[];
			total: number;
			page: number;
			pageCount: number;
			search: string;
			size: number;
			predictions: StoreIngredientPredictionsData | null;
			predictionsMessage?: string;
		};
	} = $props();

	// Server-driven search & pagination
	let searchQuery = $state(data.search || '');
	let activeSearchQuery = $state(data.search || '');
	let pageSize = $state(data.size || 10);
	let showSearch = $state(true);
	let isRefreshingPredictions = $state(false);
	let predictionError = $state('');
	let predictionSuccess = $state('');

	const predictions = $derived.by(() => data.predictions ?? null);
	const predictionIngredients = $derived.by(() => predictions?.ingredients ?? []);
	const hasPredictionIngredients = $derived.by(() => predictionIngredients.length > 0);

	// Delete confirmation dialog state
	let showDeleteDialog = $state(false);
	let stockToDelete = $state<IngredientStock | null>(null);
	let isDeleting = $state(false);

	// Server-driven pagination values
	const filteredItems = $derived.by(() => data.items || []);
	const totalPages = $derived.by(() => data.pageCount || 1);
	const currentDisplayPage = $derived.by(() => data.page || 1);
	const startIndex = $derived.by(() => (data.items?.length ?? 0) > 0 ? (data.page - 1) * data.size + 1 : 0);
	const endIndex = $derived.by(() => (data.items?.length ?? 0) > 0 ? Math.min(startIndex + (data.items?.length ?? 0) - 1, data.total) : 0);

	// Pagination function
	function go(page: number) {
		const safePage = Math.max(1, Math.min(page, totalPages));
		const s = activeSearchQuery ? `&search=${encodeURIComponent(activeSearchQuery)}` : '';
		const sizeParam = `&size=${pageSize}`;
		goto(`/master/ingredient-stocks?page=${safePage}${sizeParam}${s}`);
	}

	// Delete function
	function handleDelete(uuid: string) {
		const stock = data.items.find((i) => i.uuid === uuid);
		if (stock) {
			stockToDelete = stock;
			showDeleteDialog = true;
		}
	}

	async function confirmDelete() {
		if (!stockToDelete) return;

		isDeleting = true;
		try {
			const formData = new FormData();
			formData.append('uuid', stockToDelete.uuid);

			const response = await fetch('?/delete', {
				method: 'POST',
				body: formData
			});

				if (response.ok) {
					showDeleteDialog = false;
					stockToDelete = null;
				} else {
					alert(t('delete_failed_stock'));
				}
			} catch (error) {
				console.error('Error deleting ingredient stock:', error);
				alert(t('delete_error_stock'));
			} finally {
				isDeleting = false;
			}
	}

	function cancelDelete() {
		showDeleteDialog = false;
		stockToDelete = null;
	}

	// Search function
	function handleSearch(query: string) {
		activeSearchQuery = query;
		const sizeParam = `&size=${pageSize}`;
		goto(`/master/ingredient-stocks?page=1${sizeParam}${query ? `&search=${encodeURIComponent(query)}` : ''}`);
	}

	function clearSearch() {
		searchQuery = '';
		activeSearchQuery = '';
		goto(`/master/ingredient-stocks?page=1&size=${pageSize}`);
	}

	function handleAddIngredientStock() {
		goto('/master/ingredient-stocks/new');
	}

	function handleEditIngredientStock(uuid: string) {
		goto(`/master/ingredient-stocks/${uuid}/edit`);
	}

	// Format timestamp to readable date
	function formatDate(timestamp: number): string {
		return new Date(timestamp).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function formatGeneratedAt(timestamp?: number): string {
		if (typeof timestamp !== 'number' || Number.isNaN(timestamp) || timestamp <= 0) {
			return '-';
		}

		return new Date(timestamp).toLocaleString('id-ID', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	const columns = [
		{ key: 'ingredient', label: t('nav_ingredients') },
		{ key: 'total_quantity', label: t('col_total_quantity') },
		{ key: 'total_value', label: t('col_total_value') },
		{ key: 'current_cost', label: t('col_current_cost') },
		{ key: 'avg_cost', label: t('col_avg_cost') },
		{ key: 'created_at', label: t('col_created_at') },
		{ key: 'actions', label: t('col_actions') }
	] as const;

	function changeSize(newSize: number) {
		pageSize = newSize;
		const s = activeSearchQuery ? `&search=${encodeURIComponent(activeSearchQuery)}` : '';
		goto(`/master/ingredient-stocks?page=1&size=${pageSize}${s}`);
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

	function formatProbability(value?: number): string {
		if (typeof value !== 'number' || Number.isNaN(value)) {
			return '-';
		}
		return `${Math.round(value * 100)}%`;
	}

	function formatDecimal(value?: number | string, fractionDigits = 2): string {
		if (value === null || value === undefined || value === '') {
			return '-';
		}

		const numericValue =
			typeof value === 'number'
				? value
				: Number.parseFloat(String(value).replace(',', '.'));

		if (!Number.isFinite(numericValue)) {
			return '-';
		}

		return numericValue.toLocaleString('id-ID', {
			minimumFractionDigits: fractionDigits,
			maximumFractionDigits: fractionDigits
		});
	}

	function formatWeatherValue(value?: number): string {
		if (typeof value !== 'number' || Number.isNaN(value)) {
			return '-';
		}

		return value.toFixed(1);
	}

	function formatPercentage(value?: number | string, fractionDigits = 0): string {
		if (value === null || value === undefined || value === '') {
			return '-';
		}

		const numericValue =
			typeof value === 'number'
				? value
				: Number.parseFloat(String(value).replace(',', '.'));

		if (!Number.isFinite(numericValue)) {
			return '-';
		}

		return `${numericValue.toLocaleString('id-ID', {
			minimumFractionDigits: fractionDigits,
			maximumFractionDigits: fractionDigits
		})}%`;
	}

	function formatQuantityWithUnit(
		value?: number | string,
		...unitCandidates: Array<string | null | undefined>
	) {
		const quantity = formatDecimal(value);
		if (quantity === '-') {
			return '-';
		}

		const unitLabel = resolveUnitLabel(...unitCandidates);
		return unitLabel ? `${quantity} ${unitLabel}` : quantity;
	}

	async function refreshPredictions() {
		if (isRefreshingPredictions) {
			return;
		}

		isRefreshingPredictions = true;
		predictionError = '';
		predictionSuccess = '';

		try {
			const response = await fetch('?/predict', {
				method: 'POST'
			});

			const payload = await response.json().catch(() => ({}));

			if (!response.ok) {
				predictionError =
					payload?.message ?? 'Gagal memuat rekomendasi bahan baku. Mohon coba lagi.';
				return;
			}

			predictionSuccess =
				payload?.message ?? 'Rekomendasi bahan baku berhasil diperbarui.';
			await invalidateAll();
		} catch (error) {
			console.error('Error refreshing ingredient predictions:', error);
			predictionError =
				error instanceof Error
					? error.message
					: 'Gagal memuat rekomendasi bahan baku. Mohon coba lagi.';
		} finally {
			isRefreshingPredictions = false;
		}
	}
</script>

<section class="space-y-4">
	<h1 class="text-xl font-semibold">{t('ingredient_stocks_title')}</h1>
	<div class="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
		<div class="flex items-center gap-2">
			{#if showSearch}
				<SearchBox value={searchQuery} onSearch={handleSearch} />
				{#if activeSearchQuery}
					<Button label={t('clear')} color="gray" onClick={clearSearch} />
					<span class="text-sm text-gray-600 dark:text-gray-400">
						{t('showing_filtered_ingredient_stocks', { shown: data.items.length, total: data.total, query: activeSearchQuery })}
					</span>
				{:else}
					<span class="text-sm text-gray-600 dark:text-gray-400">{t('press_enter_to_search')}</span>
				{/if}
			{:else}
				<span class="text-sm text-gray-600 dark:text-gray-400">
					{t('showing_ingredient_stocks', { count: data.total })}
				</span>
			{/if}
		</div>
		<Button label={t('add_ingredient_stock')} color="emerald" onClick={handleAddIngredientStock} />
	</div>

	<div class="overflow-x-auto rounded-[var(--radius-card)] border bg-white/60 backdrop-blur-md dark:bg-white/10" style="border-color: var(--ui-border);">
		<table class="min-w-full text-sm">
			<thead class="bg-gray-50 dark:bg-gray-900">
				<tr>
					{#each columns as c}
						<th class="px-4 py-2 text-left font-semibold">{c.label}</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each data.items as row (row.uuid)}
					<tr class="odd:bg-gray-50 dark:odd:bg-gray-800">
						<td class="px-4 py-2 font-medium">{row.ingredient.name}</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">
							{formatQuantityDisplay(
								row.total_quantity,
								row.unit_of_measure_code,
								row.ingredient?.base_uom?.code,
								row.unit_of_measure_name,
								row.ingredient?.base_uom?.name
							)}
						</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">{formatCurrency(row.total_value)}</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">{formatCurrency(row.current_cost)}</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">{formatCurrency(row.avg_cost)}</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">{formatDate(row.created_at)}</td>
						<td class="px-4 py-2">
							<div class="flex items-center gap-2">
								<Button label={t('edit')} color="violet" onClick={() => handleEditIngredientStock(row.uuid)} />
								<Button label={t('delete')} color="red" onClick={() => handleDelete(row.uuid)} />
							</div>
						</td>
					</tr>
				{/each}

				{#if data.items.length === 0}
					<tr>
						<td class="px-4 py-6 opacity-60" colspan={columns.length}>
							{activeSearchQuery ? t('no_ingredient_stocks_match_search') : t('no_ingredient_stocks_available')}
						</td>
					</tr>
				{/if}
			</tbody>
		</table>
	</div>

	<div class="mt-3">
		<TablePagination
			page={currentDisplayPage}
			pageCount={totalPages}
			total={data.total}
			start={startIndex}
			end={endIndex}
			size={pageSize}
			sizes={[3, 5, 10, 25, 50]}
			onChangePage={go}
			onChangeSize={changeSize}
		/>
	</div>

	<div class="mt-8 space-y-4">
		<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
			<div>
				<h2 class="text-lg font-semibold">Store Ingredient Predictions</h2>
				{#if data.predictionsMessage}
					<p class="text-sm text-gray-600 dark:text-gray-400">{data.predictionsMessage}</p>
				{/if}
			</div>
			<Button
				label={isRefreshingPredictions ? 'Refreshing...' : 'Refresh Recommendations'}
				color="violet"
				disabled={isRefreshingPredictions}
				onClick={refreshPredictions}
			/>
		</div>

		{#if predictionError}
			<div class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900/40 dark:bg-red-900/10 dark:text-red-300">
				{predictionError}
			</div>
		{/if}

		{#if predictionSuccess}
			<div class="rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700 dark:border-emerald-900/40 dark:bg-emerald-900/10 dark:text-emerald-300">
				{predictionSuccess}
			</div>
		{/if}

		{#if predictions}
			<div
				class="space-y-4 rounded-[var(--radius-card)] border bg-white/60 p-4 backdrop-blur-md dark:bg-white/10"
				style="border-color: var(--ui-border);"
			>
				<div class="grid gap-4 md:grid-cols-2">
					<div class="space-y-2">
						<p class="text-sm font-medium uppercase tracking-wide text-gray-500 dark:text-gray-400">
							Generated At
						</p>
						<p class="text-base">{formatGeneratedAt(predictions.generated_at_ms)}</p>
						{#if predictions.llm_summary}
							<p class="text-base leading-relaxed text-gray-700 dark:text-gray-200">
								{predictions.llm_summary}
							</p>
						{/if}
						{#if predictions.llm_model}
							<p class="text-sm text-gray-500 dark:text-gray-500">Model: {predictions.llm_model}</p>
						{/if}
					</div>
					<div class="rounded-lg bg-gray-50 p-4 dark:bg-gray-900/40">
						<p class="text-sm font-semibold text-gray-700 dark:text-gray-200">Weather</p>
						<div class="mt-2 grid grid-cols-2 gap-2 text-sm text-gray-600 dark:text-gray-300">
							<div>
								<p class="font-medium text-gray-700 dark:text-gray-200">Summary</p>
								<p>{predictions.weather?.summary ?? '-'}</p>
							</div>
							<div>
								<p class="font-medium text-gray-700 dark:text-gray-200">Temp Min (&deg;C)</p>
								<p>{formatWeatherValue(predictions.weather?.temp_min_c)}</p>
							</div>
							<div>
								<p class="font-medium text-gray-700 dark:text-gray-200">Temp Max (&deg;C)</p>
								<p>{formatWeatherValue(predictions.weather?.temp_max_c)}</p>
							</div>
							<div>
								<p class="font-medium text-gray-700 dark:text-gray-200">Humidity (%)</p>
								<p>{formatWeatherValue(predictions.weather?.humidity_avg)}</p>
							</div>
							<div>
								<p class="font-medium text-gray-700 dark:text-gray-200">Precip. (mm)</p>
								<p>{formatWeatherValue(predictions.weather?.precipitation_total_mm)}</p>
							</div>
						</div>
					</div>
				</div>

				<div>
					<h3 class="text-base font-semibold text-gray-700 dark:text-gray-200">
						Restock Recommendations
					</h3>

					{#if hasPredictionIngredients}
						<div class="mt-3 overflow-x-auto">
							<table class="min-w-full text-sm">
								<thead>
									<tr class="text-left text-gray-500 dark:text-gray-400">
										<th class="px-3 py-2 font-semibold">Ingredient</th>
										<th class="px-3 py-2 font-semibold">Unit</th>
										<th class="px-3 py-2 font-semibold">Restock</th>
										<th class="px-3 py-2 font-semibold">Probability</th>
										<th class="px-3 py-2 font-semibold">Recommended Qty</th>
										<th class="px-3 py-2 font-semibold">Current Qty</th>
										<th class="px-3 py-2 font-semibold">Minimum Qty</th>
										<th class="px-3 py-2 font-semibold">Error Margin</th>
										<th class="px-3 py-2 font-semibold">Reasoning</th>
									</tr>
								</thead>
								<tbody>
									{#each predictionIngredients as item, index (item.ingredient_catalog_uuid ?? index.toString())}
										<tr class="border-b last:border-b-0 dark:border-gray-800/60">
											<td class="px-3 py-2 text-sm font-medium">
												{item.ingredient_name ?? '-'}
											</td>
											<td class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400">
												{resolveUnitLabel(item.unit_of_measure_name, item.unit_of_measure_code)}
											</td>
											<td class="px-3 py-2">
												<span class="rounded-full bg-amber-100 px-2 py-1 text-sm font-medium text-amber-800 dark:bg-amber-900/40 dark:text-amber-200">
													{item.restock_label ?? '-'}
												</span>
											</td>
											<td class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400">
												{formatProbability(item.restock_probability)}
											</td>
											<td class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400">
												{formatQuantityWithUnit(
													item.recommended_restock_qty,
													item.unit_of_measure_name,
													item.unit_of_measure_code
												)}
											</td>
											<td class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400">
												{formatQuantityWithUnit(
													item.current_stock_qty,
													item.unit_of_measure_name,
													item.unit_of_measure_code
												)}
											</td>
											<td class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400">
												{formatQuantityWithUnit(
													item.minimum_stock_qty,
													item.unit_of_measure_name,
													item.unit_of_measure_code
												)}
											</td>
											<td class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400">
												{formatPercentage(item.forecast_error_margin_pct, 1)}
											</td>
											<td class="px-3 py-2 text-base leading-relaxed text-gray-700 dark:text-gray-200">
												{item.llm_reasoning ?? '-'}
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{:else}
						<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
							Tidak ada rekomendasi bahan baku dalam hasil prediksi.
						</p>
					{/if}
				</div>
			</div>
		{:else}
			<p class="text-sm text-gray-600 dark:text-gray-400">
				Belum ada hasil rekomendasi bahan baku untuk store ini.
			</p>
		{/if}
	</div>
</section>

<ConfirmDialog
	show={showDeleteDialog}
	title={t('delete_confirm_title_stock')}
	message={t('delete_confirm_message_stock')}
	confirmText={t('delete')}
	cancelText={t('cancel')}
	isLoading={isDeleting}
	onConfirm={confirmDelete}
	onCancel={cancelDelete}
/>
