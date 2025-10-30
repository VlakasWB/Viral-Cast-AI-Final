<script lang="ts">
	import SearchBox from '$lib/components/SearchBox.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import TablePagination from '$lib/components/TablePagination.svelte';
	import type { IngredientCatalog } from '$lib/types/ingredient-catalog.js';
import { goto } from '$app/navigation';
import { page } from '$app/state';
import { t } from '$lib/stores/i18n';

	let {
		data
}: {
    data: {
        items: IngredientCatalog[];
        total: number;
        page: number;
        pageCount: number;
        search: string;
        size: number;
    };
} = $props();

	// Server-driven search & pagination
let searchQuery = $state(data.search || '');
let activeSearchQuery = $state(data.search || '');
	let pageSize = $state(data.size || 10);
	let showSearch = $state(true);

	// Delete confirmation dialog state
	let showDeleteDialog = $state(false);
let ingredientToDelete = $state<IngredientCatalog | null>(null);
	let isDeleting = $state(false);

	const basePath = '/master/ingredient-catalog';

	// Server-driven pagination values
	const filteredItems = $derived.by(() => data.items || []);
	const totalPages = $derived.by(() => data.pageCount || 1);
	const currentDisplayPage = $derived.by(() => data.page || 1);
	const startIndex = $derived.by(() => (data.items?.length ?? 0) > 0 ? (data.page - 1) * data.size + 1 : 0);
	const endIndex = $derived.by(() => (data.items?.length ?? 0) > 0 ? Math.min(startIndex + (data.items?.length ?? 0) - 1, data.total) : 0);

	// Pagination function
	const navigateWithParams = (targetPage: number, size = pageSize, search = activeSearchQuery) => {
		const params = new URLSearchParams();
		params.set('page', String(targetPage));
		params.set('size', String(size));
		if (search) {
			params.set('search', search);
		}
		const query = params.toString();
		goto(query ? `${basePath}?${query}` : basePath);
	};

	function go(pageNumber: number) {
		navigateWithParams(pageNumber);
	}

	// Delete function
	function handleDelete(uuid: string) {
		const ingredient = data.items.find((i) => i.uuid === uuid);
		if (ingredient) {
			ingredientToDelete = ingredient;
			showDeleteDialog = true;
		}
	}

	async function confirmDelete() {
		if (!ingredientToDelete) return;

		isDeleting = true;
		try {
			const formData = new FormData();
			formData.append('uuid', ingredientToDelete.uuid);

			const response = await fetch('?/delete', {
				method: 'POST',
				body: formData
			});

			if (response.ok) {
				showDeleteDialog = false;
				ingredientToDelete = null;
			} else {
				alert('Failed to delete ingredient');
			}
		} catch (error) {
			console.error('Error deleting ingredient:', error);
			alert('Error deleting ingredient');
		} finally {
			isDeleting = false;
		}
	}

	function cancelDelete() {
		showDeleteDialog = false;
		ingredientToDelete = null;
	}

	// Search function
function handleSearch(query: string) {
	activeSearchQuery = query;
	searchQuery = query;
	navigateWithParams(1, pageSize, query);
}

	function clearSearch() {
		searchQuery = '';
		activeSearchQuery = '';
		navigateWithParams(1, pageSize, '');
	}

	function handleAddIngredient() {
		const currentSearch = page?.url?.search ?? '';
		const redirectTarget = `${basePath}${currentSearch}`;
		goto(`${basePath}/new?redirectTo=${encodeURIComponent(redirectTarget)}`);
	}

	function handleEditIngredient(uuid: string) {
		const currentSearch = page?.url?.search ?? '';
		const redirectTarget = `${basePath}${currentSearch}`;
		goto(`${basePath}/${uuid}/edit?redirectTo=${encodeURIComponent(redirectTarget)}`);
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

	const columns = [
		{ key: 'name', label: t('col_name') },
		{ key: 'base_uom', label: t('col_base_uom') },
		{ key: 'minimum_stock', label: t('col_minimum_stock') },
		{ key: 'shelf_life_days', label: t('col_shelf_life_days') },
		{ key: 'created_at', label: t('col_created_at') },
		{ key: 'actions', label: t('col_actions') }
	] as const;

	function changeSize(newSize: number) {
		pageSize = newSize;
		navigateWithParams(1, newSize, activeSearchQuery);
	}
</script>

<section class="space-y-4">
	<h1 class="text-xl font-semibold">{t('ingredients_title')}</h1>
	<div class="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
		<div class="flex items-center gap-2">
			{#if showSearch}
				<SearchBox value={searchQuery} onSearch={handleSearch} />
				{#if activeSearchQuery}
					<Button label={t('clear')} color="gray" onClick={clearSearch} />
					<span class="text-sm text-gray-600 dark:text-gray-400">
						{t('showing_filtered_ingredients', { shown: data.items.length, total: data.total, query: activeSearchQuery })}
					</span>
				{:else}
					<span class="text-sm text-gray-600 dark:text-gray-400">{t('press_enter_to_search')}</span>
				{/if}
			{:else}
				<span class="text-sm text-gray-600 dark:text-gray-400">
					{t('showing_ingredients', { count: data.total })}
				</span>
			{/if}
		</div>
		<Button label={t('add_ingredient')} color="emerald" onClick={handleAddIngredient} />
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
						<td class="px-4 py-2 font-medium">{row.name}</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">
							{row.base_uom.name} ({row.base_uom.code})
						</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">{row.minimum_stock}</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">{row.shelf_life_days}</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">{formatDate(row.created_at)}</td>
						<td class="px-4 py-2">
							<div class="flex items-center gap-2">
								<Button label={t('edit')} color="violet" onClick={() => handleEditIngredient(row.uuid)} />
								<Button label={t('delete')} color="red" onClick={() => handleDelete(row.uuid)} />
							</div>
						</td>
					</tr>
				{/each}

				{#if data.items.length === 0}
					<tr>
						<td class="px-4 py-6 opacity-60" colspan={columns.length}>
							{activeSearchQuery ? t('no_ingredients_match_search') : t('no_ingredients_available')}
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
</section>

<ConfirmDialog
	show={showDeleteDialog}
	title={t('delete_confirm_title_ingredient')}
	message={t('delete_confirm_message_ingredient')}
	confirmText={t('delete')}
	cancelText={t('cancel')}
	isLoading={isDeleting}
	onConfirm={confirmDelete}
	onCancel={cancelDelete}
/>
