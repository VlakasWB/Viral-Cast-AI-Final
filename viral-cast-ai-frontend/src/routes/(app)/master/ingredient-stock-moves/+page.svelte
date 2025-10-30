<script lang="ts">
	import SearchBox from '$lib/components/SearchBox.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import TablePagination from '$lib/components/TablePagination.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import type { IngredientStockMove } from '$lib/types/ingredient-stock-moves';
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import { t } from '$lib/stores/i18n';

	let {
		data
	}: {
		data: {
			stockMoves: IngredientStockMove[];
			total: number;
			page: number;
			pageCount: number;
			search: string;
			size: number;
			name: string;
			move_type: string;
			ref_uuid: string;
			from_date: string;
			to_date: string;
			sort_by: string;
			sort_direction: string;
			ingredients: any[];
		};
	} = $props();

	function toDateInputValue(value?: string | number | null) {
		if (value === null || value === undefined) return '';
		const raw = String(value).trim();
		if (!raw) return '';
		if (/^\d{4}-\d{2}-\d{2}$/.test(raw)) return raw;
		const numeric = Number(raw);
		const timestamp = Number.isFinite(numeric) && raw.length >= 6 ? numeric : Date.parse(raw);
		if (!Number.isFinite(timestamp)) return '';
		const date = new Date(timestamp);
		if (Number.isNaN(date.getTime())) return '';
		return date.toISOString().slice(0, 10);
	}

	const SORT_FIELDS: Record<string, string> = {
		no: 'created_at',
		ingredient: 'ingredient_name',
		move_type: 'move_type',
		quantity: 'quantity',
		expiry_at: 'expiry_at',
		created_at: 'created_at'
	};

	// ID: State untuk pencarian dan filter
	// EN: State for search and filters
	let searchQuery = $state(data.search || '');
	let activeSearchQuery = $state(data.search || '');
	let pageSize = $state(data.size || 10);
	let selectedIngredient = $state(data.name || '');
	let selectedMoveType = $state(data.move_type || '');
	let fromDate = $state(toDateInputValue(data.from_date));
	let toDate = $state(toDateInputValue(data.to_date));
	let sortBy = $state(data.sort_by || '');
	let sortDirection = $state(data.sort_direction || '');
	let refUuid = $state(data.ref_uuid || '');
	let showSearch = $state(true);

	// ID: State untuk dialog konfirmasi hapus
	// EN: State for delete confirmation dialog
	let showDeleteDialog = $state(false);
	let stockMoveToDelete = $state<IngredientStockMove | null>(null);
	let isDeleting = $state(false);

	// ID: Nilai pagination yang dikelola server
	// EN: Server-driven pagination values
	const filteredItems = $derived.by(() => data.stockMoves || []);
	const totalPages = $derived.by(() => data.pageCount || 1);
	const currentDisplayPage = $derived.by(() => data.page || 1);
	const startIndex = $derived.by(() =>
		(data.stockMoves?.length ?? 0) > 0 ? (data.page - 1) * data.size + 1 : 0
	);
	const endIndex = $derived.by(() =>
		(data.stockMoves?.length ?? 0) > 0
			? Math.min(startIndex + (data.stockMoves?.length ?? 0) - 1, data.total)
			: 0
	);

	// ID: Opsi untuk dropdown move type
	// EN: Options for move type dropdown
	const moveTypeOptions = [
		{ value: '', label: t('filter_all_types') },
		{ value: 'PRODUCTION', label: t('move_type_production') },
		{ value: 'ADJUSTMENT', label: t('move_type_adjustment') },
		{ value: 'WASTE', label: t('move_type_waste') },
		{ value: 'RETURN', label: t('move_type_return') },
		{ value: 'PURCHASE', label: t('move_type_purchase') }
	];

	const ingredientFilterOptions = $derived.by(() => {
		const options = [{ value: '', label: t('filter_all_ingredients') }];
		const seen = new Set<string>();

		for (const ingredient of data.ingredients ?? []) {
			const name = typeof ingredient?.name === 'string' ? ingredient.name.trim() : '';
			if (!name || seen.has(name)) continue;
			seen.add(name);
			options.push({ value: name, label: name });
		}

		return options;
	});

	const ingredientNameMap = $derived.by(() => {
		const map = new Map<string, string>();
		for (const ingredient of data.ingredients ?? []) {
			if (ingredient?.uuid) {
				map.set(ingredient.uuid, ingredient.name);
			}
		}
		return map;
	});

	function extractIngredientName(raw?: string | null) {
		if (!raw) return '';
		const trimmed = raw.trim();
		if (!trimmed) return '';
		const parts = trimmed.split(' - ').map((part) => part.trim());
		if (parts.length >= 2) {
			// Many payloads follow "TYPE - NAME - UUID"
			return parts[1];
		}
		return trimmed;
	}

	function resolveIngredientName(stockMove: IngredientStockMove) {
		const directName = stockMove.ingredient?.name;
		if (directName) return directName;

		const lookupKey =
			stockMove.ingredient_catalog_uuid ||
			stockMove.ingredient_uuid ||
			stockMove.ingredient?.uuid ||
			'';
		if (lookupKey) {
			const mapped = ingredientNameMap.get(lookupKey);
			if (mapped) return mapped;
		}

		const extracted = extractIngredientName(stockMove.name);
		return extracted || '-';
	}

	// ID: Fungsi untuk navigasi pagination
	// EN: Pagination navigation function
	function getSortField(columnKey: string) {
		return SORT_FIELDS[columnKey] ?? columnKey;
	}

	function getSortDirection(columnKey: string) {
		const field = getSortField(columnKey);
		return sortBy === field ? sortDirection : '';
	}

	function buildQueryParams(pageValue: number, sizeValue: number) {
		const params = new URLSearchParams();
		params.set('page', Math.max(1, pageValue).toString());
		params.set('size', Math.max(1, sizeValue).toString());

		if (activeSearchQuery) params.set('search', activeSearchQuery);
		if (selectedIngredient) params.set('name', selectedIngredient);
		if (selectedMoveType) params.set('move_type', selectedMoveType);
		if (refUuid) params.set('ref_uuid', refUuid);
		if (fromDate) params.set('from_date', fromDate);
		if (toDate) params.set('to_date', toDate);
		if (sortBy) params.set('sort_by', sortBy);
		if (sortDirection) params.set('sort_direction', sortDirection);

		return params;
	}

	function navigateWithCurrentState(
		pageValue: number,
		options: { syncSearch?: boolean; sizeValue?: number } = {}
	) {
		if (options.syncSearch) {
			activeSearchQuery = searchQuery.trim();
		}

		const params = buildQueryParams(pageValue, options.sizeValue ?? pageSize);
		goto(`/master/ingredient-stock-moves?${params.toString()}`);
	}

	function go(page: number) {
		const safePage = Math.max(1, Math.min(page, totalPages));
		navigateWithCurrentState(safePage);
	}

	function changePageSize(newSize: number) {
		pageSize = newSize;
		navigateWithCurrentState(1, { sizeValue: pageSize });
	}

	// ID: Fungsi untuk menangani pencarian
	// EN: Function to handle search
	function handleSearch(query: string) {
		const trimmed = query.trim();
		searchQuery = trimmed;
		activeSearchQuery = trimmed;
		navigateWithCurrentState(1, { syncSearch: true });
	}

	// ID: Fungsi untuk menerapkan filter
	// EN: Function to apply filters
	function applyFilters() {
		navigateWithCurrentState(1, { syncSearch: true });
	}

	// ID: Fungsi untuk reset filter
	// EN: Function to reset filters
	function resetFilters() {
		searchQuery = '';
		activeSearchQuery = '';
		selectedIngredient = '';
		selectedMoveType = '';
		fromDate = '';
		toDate = '';
		sortBy = '';
		sortDirection = '';
		refUuid = '';
		navigateWithCurrentState(1);
	}

	function toggleSort(columnKey: string) {
		const field = getSortField(columnKey);
		const isSameField = sortBy === field;
		const nextDirection = isSameField && sortDirection === 'asc' ? 'desc' : 'asc';
		sortBy = field;
		sortDirection = nextDirection;
		navigateWithCurrentState(1);
	}

	// ID: Fungsi untuk menangani hapus
	// EN: Function to handle delete
	function handleDelete(uuid: string) {
		const stockMove = data.stockMoves.find((sm) => sm.uuid === uuid);
		if (stockMove) {
			stockMoveToDelete = stockMove;
			showDeleteDialog = true;
		}
	}

	// ID: Fungsi untuk konfirmasi hapus
	// EN: Function to confirm delete
	function confirmDelete() {
		if (stockMoveToDelete) {
			isDeleting = true;
			// Form akan disubmit melalui enhance
		}
	}

	// ID: Fungsi untuk format tanggal
	// EN: Function to format date
	function formatDate(dateValue?: string | number | null) {
		if (!dateValue && dateValue !== 0) return '-';
		const date =
			typeof dateValue === 'number'
				? new Date(dateValue)
				: new Date(dateValue ?? '');
		if (Number.isNaN(date.getTime())) return '-';
		return date.toLocaleDateString('id-ID', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// ID: Fungsi untuk format tipe move
	// EN: Function to format move type
	function formatMoveType(type: string) {
		const labels: Record<string, string> = {
			PRODUCTION: t('move_type_production'),
			ADJUSTMENT: t('move_type_adjustment'),
			WASTE: t('move_type_waste'),
			RETURN: t('move_type_return'),
			PURCHASE: t('move_type_purchase')
		};
		return labels[type] || type;
	}

	// ID: Fungsi untuk mendapatkan class badge berdasarkan tipe
	// EN: Function to get badge class based on type
	function getMoveTypeBadgeClass(type: string) {
		const classes: Record<string, string> = {
			PRODUCTION: 'bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-300',
			ADJUSTMENT: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
			WASTE: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300',
			RETURN: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300',
			PURCHASE: 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-300'
		};
		return classes[type] || 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300';
	}

	function viewStockMove(uuid: string) {
		goto(`/master/ingredient-stock-moves/${uuid}`);
	}

	function editStockMove(uuid: string) {
		goto(`/master/ingredient-stock-moves/${uuid}/edit`);
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
	<title>{t('ingredient_stock_moves_title')} - Viral Cast AI</title>
</svelte:head>

<div class="p-6">
	<!-- ID: Header dengan tombol tambah / EN: Header with add button -->
	<div class="mb-6 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
			<h1 class="text-2xl font-bold text-gray-900 dark:text-white">{t('ingredient_stock_moves_title')}</h1>
			<p class="text-gray-600 dark:text-gray-400">{t('ingredient_stock_moves_subtitle')}</p>
        </div>
		<Button
			label={t('add_ingredient_stock_move') ?? 'Add Stock Move'}
			color="emerald"
			onClick={() => goto('/master/ingredient-stock-moves/create')}
		/>
	</div>

	<!-- ID: Filter dan pencarian modern / EN: Modern filters and search -->
	<div
		class="mb-8 rounded-xl border border-gray-300/50 bg-white/80 p-6 shadow-lg backdrop-blur-sm dark:border-gray-600/50 dark:bg-gray-800/80"
	>
		<div class="space-y-5">
			<!-- Search full width on its own row -->
			<div>
				<label class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
					{t('stock_move_search_label')}
				</label>
				<SearchBox
					value={searchQuery}
					placeholder={t('stock_move_search_placeholder')}
					onSearch={handleSearch}
					class="w-full"
				/>
			</div>

			<!-- Ingredient & move type aligned -->
			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				<div>
					<Select
						id="ingredient-filter"
						label={t('nav_ingredients')}
						bind:value={selectedIngredient}
						options={ingredientFilterOptions}
						placeholder={t('filter_all_ingredients')}
						on:change={applyFilters}
						searchable
						searchPlaceholder={t('stock_move_search_placeholder')}
					/>
				</div>

				<div>
					<Select
						id="move-type-filter"
						label={t('move_type')}
						bind:value={selectedMoveType}
						options={moveTypeOptions}
						placeholder={t('filter_all_types')}
						on:change={applyFilters}
					/>
				</div>
			</div>

			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				<div>
					<label class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
						{t('stock_move_from_date')}
					</label>
					<input
						type="date"
						class="form-input w-full"
						bind:value={fromDate}
						max={toDate || undefined}
						on:change={applyFilters}
					/>
				</div>

				<div>
					<label class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
						{t('stock_move_to_date')}
					</label>
					<input
						type="date"
						class="form-input w-full"
						bind:value={toDate}
						min={fromDate || undefined}
						on:change={applyFilters}
					/>
				</div>
			</div>

			<div class="flex flex-col gap-2 sm:flex-row sm:justify-end">
				<Button label={t('clear')} color="gray" size="sm" onClick={resetFilters} />
				<Button label={t('apply_filters')} color="orange" size="sm" onClick={applyFilters} />
			</div>
		</div>
	</div>

	<!-- ID: Tabel data / EN: Data table -->
	<div
		class="rounded-lg border border-gray-200 bg-white shadow-sm dark:border-gray-700 dark:bg-gray-800"
	>
		<div class="overflow-x-auto">
			<table class="w-full text-left text-sm text-gray-500 dark:text-gray-400">
				<thead
					class="bg-gray-50 text-xs text-gray-700 uppercase dark:bg-gray-700 dark:text-gray-400"
				>
					<tr>
						<th scope="col" class="px-6 py-3">
							<button
								type="button"
								class="flex items-center gap-1"
								on:click={() => toggleSort('no')}
								aria-label="Sort by number"
							>
								<span>NO</span>
								{#if getSortDirection('no')}
									<svg
										class={`h-3 w-3 stroke-current transition-transform ${getSortDirection('no') === 'desc' ? 'transform rotate-180' : ''}`}
										viewBox="0 0 20 20"
										fill="none"
										xmlns="http://www.w3.org/2000/svg"
									>
										<path
											d="M6 12l4-4 4 4"
											stroke-width="2"
											stroke-linecap="round"
											stroke-linejoin="round"
										/>
									</svg>
								{/if}
							</button>
						</th>
						<th scope="col" class="px-6 py-3">
							<button
								type="button"
								class="flex items-center gap-1"
								on:click={() => toggleSort('ingredient')}
								aria-label="Sort by ingredient"
							>
								<span>{t('nav_ingredients')}</span>
								{#if getSortDirection('ingredient')}
									<svg
										class={`h-3 w-3 stroke-current transition-transform ${getSortDirection('ingredient') === 'desc' ? 'transform rotate-180' : ''}`}
										viewBox="0 0 20 20"
										fill="none"
										xmlns="http://www.w3.org/2000/svg"
									>
										<path
											d="M6 12l4-4 4 4"
											stroke-width="2"
											stroke-linecap="round"
											stroke-linejoin="round"
										/>
									</svg>
								{/if}
							</button>
						</th>
						<th scope="col" class="px-6 py-3">
							<button
								type="button"
								class="flex items-center gap-1"
								on:click={() => toggleSort('move_type')}
								aria-label="Sort by move type"
							>
								<span>{t('move_type')}</span>
								{#if getSortDirection('move_type')}
									<svg
										class={`h-3 w-3 stroke-current transition-transform ${getSortDirection('move_type') === 'desc' ? 'transform rotate-180' : ''}`}
										viewBox="0 0 20 20"
										fill="none"
										xmlns="http://www.w3.org/2000/svg"
									>
										<path
											d="M6 12l4-4 4 4"
											stroke-width="2"
											stroke-linecap="round"
											stroke-linejoin="round"
										/>
									</svg>
								{/if}
							</button>
						</th>
						<th scope="col" class="px-6 py-3">
							<button
								type="button"
								class="flex items-center gap-1"
								on:click={() => toggleSort('quantity')}
								aria-label="Sort by quantity"
							>
								<span>{t('quantity')}</span>
								{#if getSortDirection('quantity')}
									<svg
										class={`h-3 w-3 stroke-current transition-transform ${getSortDirection('quantity') === 'desc' ? 'transform rotate-180' : ''}`}
										viewBox="0 0 20 20"
										fill="none"
										xmlns="http://www.w3.org/2000/svg"
									>
										<path
											d="M6 12l4-4 4 4"
											stroke-width="2"
											stroke-linecap="round"
											stroke-linejoin="round"
										/>
									</svg>
								{/if}
							</button>
						</th>
						<th scope="col" class="px-6 py-3">
							<button
								type="button"
								class="flex items-center gap-1"
								on:click={() => toggleSort('expiry_at')}
								aria-label="Sort by expiry"
							>
								<span>Expiry</span>
								{#if getSortDirection('expiry_at')}
									<svg
										class={`h-3 w-3 stroke-current transition-transform ${getSortDirection('expiry_at') === 'desc' ? 'transform rotate-180' : ''}`}
										viewBox="0 0 20 20"
										fill="none"
										xmlns="http://www.w3.org/2000/svg"
									>
										<path
											d="M6 12l4-4 4 4"
											stroke-width="2"
											stroke-linecap="round"
											stroke-linejoin="round"
										/>
									</svg>
								{/if}
							</button>
						</th>
						<th scope="col" class="px-6 py-3">
							<button
								type="button"
								class="flex items-center gap-1"
								on:click={() => toggleSort('created_at')}
								aria-label="Sort by created at"
							>
								<span>{t('col_created_at')}</span>
								{#if getSortDirection('created_at')}
									<svg
										class={`h-3 w-3 stroke-current transition-transform ${getSortDirection('created_at') === 'desc' ? 'transform rotate-180' : ''}`}
										viewBox="0 0 20 20"
										fill="none"
										xmlns="http://www.w3.org/2000/svg"
									>
										<path
											d="M6 12l4-4 4 4"
											stroke-width="2"
											stroke-linecap="round"
											stroke-linejoin="round"
										/>
									</svg>
								{/if}
							</button>
						</th>
						<th scope="col" class="px-6 py-3">{t('col_actions')}</th>
					</tr>
				</thead>
				<tbody>
					{#each filteredItems as stockMove, index (stockMove.uuid)}
					<tr
							class="border-b bg-white/95 transition-colors duration-200 hover:bg-gradient-to-r hover:from-sky-50 hover:via-indigo-50 hover:to-white dark:border-gray-700 dark:bg-slate-900/60 dark:hover:from-slate-900 dark:hover:via-slate-800 dark:hover:to-slate-900"
						>
							<td class="px-6 py-4 font-medium text-gray-900 dark:text-white">
								{startIndex + index}
							</td>
							<td class="px-6 py-4">
								{resolveIngredientName(stockMove)}
							</td>
							<td class="px-6 py-4">
								<span
									class="rounded-full px-2 py-1 text-xs font-medium {getMoveTypeBadgeClass(
										stockMove.move_type
									)}"
								>
									{formatMoveType(stockMove.move_type)}
								</span>
							</td>
							<td class="px-6 py-4 font-medium">
								{formatQuantityDisplay(
									stockMove.quantity,
									stockMove.unit_of_measure_code,
									stockMove.unit_of_measure?.code,
									stockMove.unit_of_measure?.symbol,
									stockMove.unit_of_measure_name,
									stockMove.unit_of_measure?.name,
									stockMove.ingredient?.base_uom
								)}
							</td>
							<td class="px-6 py-4">
								{formatDate(stockMove.expiry_at)}
							</td>
							<td class="px-6 py-4">
								{formatDate(stockMove.created_at)}
							</td>
							<td class="px-6 py-4">
								<div class="flex items-center gap-2">
									<Button
										type="button"
										size="sm"
										color="gray"
										label={t('view')}
										onClick={() => viewStockMove(stockMove.uuid)}
									/>
									<Button
										type="button"
										size="sm"
										color="violet"
										label={t('edit')}
										onClick={() => editStockMove(stockMove.uuid)}
									/>
									<Button
										type="button"
										size="sm"
										color="red"
										label={t('delete')}
										onClick={() => handleDelete(stockMove.uuid)}
									/>
								</div>
							</td>
						</tr>
					{:else}
						<tr>
							<td colspan="7" class="px-6 py-8 text-center text-gray-500 dark:text-gray-400">
								{t('no_stock_moves_found')}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- ID: Pagination / EN: Pagination -->
		<TablePagination
			page={currentDisplayPage}
			pageCount={totalPages}
			total={data.total}
			start={startIndex}
			end={endIndex}
			size={pageSize}
			sizes={[5, 10, 25, 50]}
			onChangePage={go}
			onChangeSize={changePageSize}
		/>
	</div>
</div>

<!-- ID: Dialog konfirmasi hapus / EN: Delete confirmation dialog -->
<ConfirmDialog
	bind:show={showDeleteDialog}
	title={t('delete_confirm_title_stock_move')}
	message={t('delete_confirm_message_stock_move')}
	confirmText={t('delete')}
	cancelText={t('cancel')}
	onConfirm={confirmDelete}
	loading={isDeleting}
/>

<!-- ID: Form tersembunyi untuk hapus / EN: Hidden form for delete -->
{#if stockMoveToDelete}
	<form
		method="POST"
		action="?/delete"
		use:enhance={() => {
			return async ({ result }) => {
				isDeleting = false;
				showDeleteDialog = false;
				stockMoveToDelete = null;

				if (result.type === 'success') {
					// ID: Refresh halaman setelah berhasil hapus / EN: Refresh page after successful delete
					goto('/master/ingredient-stock-moves?page=' + data.page + '&size=' + data.size);
				}
			};
		}}
		class="hidden"
	>
		<input type="hidden" name="uuid" value={stockMoveToDelete.uuid} />
	</form>
{/if}
