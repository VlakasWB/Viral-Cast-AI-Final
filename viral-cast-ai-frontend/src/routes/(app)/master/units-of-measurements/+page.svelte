<script lang="ts">
	import { t } from '$lib/stores/i18n';
	import Pagination from '$lib/components/Pagination.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import SearchBox from '$lib/components/SearchBox.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import type { PageData, ActionData } from './$types';
	import type { UOM } from '$lib/types/uom';
	import { formatDateTime } from '$lib/utils/date';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Toaster, toast } from 'svelte-sonner';

	export let data: PageData;
	export let form: ActionData;

	// Reactive statement to show toast on form submission result
	$: if (form?.message) {
		if (form?.success) {
			toast.success(form.message);
		} else {
			toast.error(form.message);
		}
	}

	let activeSearchQuery: string = data.q ?? '';
	let searchQuery: string = data.q ?? '';
	// Columns
	const columns = [
		{ key: 'code', label: t('col_code') },
		{ key: 'name', label: t('col_name') },
		{ key: 'created_at', label: t('col_created_at') },
		{ key: 'updated_at', label: t('col_updated_at') },
		{ key: 'actions', label: t('col_actions') }
	];

	const basePath = '/master/units-of-measurements';
	const buildRedirectTarget = () => {
		const currentSearch = page?.url?.search ?? '';
		return `${basePath}${currentSearch}`;
	};

	// Pagination state from server
	let currentDisplayPage: number = data.page ?? 1;
	let totalPages: number = data.pageCount ?? 1;

	// Items directly from server response
	$: items = (data.items as UOM[]) ?? [];

	// Show counts using server-side computed totals
	$: filteredCount = items.length;
	$: totalCount = data.total ?? items.length;
	const newHref = () => `${basePath}/new?redirectTo=${encodeURIComponent(buildRedirectTarget())}`;
	const editHref = (uuid: string) =>
		`${basePath}/${uuid}/edit?redirectTo=${encodeURIComponent(buildRedirectTarget())}`;

	function go(page: number) {
		const params = new URLSearchParams();
		params.set('page', String(page));
		params.set('limit', String(data.size ?? 10));
		if (activeSearchQuery) {
			params.set('search', activeSearchQuery);
		}
		goto(`${basePath}?${params.toString()}`);
	}

	function handleSearch(q: string) {
		activeSearchQuery = q;
		searchQuery = q;
		const params = new URLSearchParams();
		params.set('page', '1');
		params.set('limit', String(data.size ?? 10));
		if (q) params.set('search', q);
		goto(`${basePath}?${params.toString()}`);
	}

	function clearSearch() {
		activeSearchQuery = '';
		searchQuery = '';
		const params = new URLSearchParams();
		params.set('page', '1');
		params.set('limit', String(data.size ?? 10));
		goto(`${basePath}?${params.toString()}`);
	}

	function onDelete(uuid: string) {
		const form = document.createElement('form');
		form.method = 'POST';
		form.action = '?/delete';
		const input = document.createElement('input');
		input.type = 'hidden';
		input.name = 'uuid';
		input.value = uuid;
		form.appendChild(input);
		document.body.appendChild(form);
		form.submit();
	}

	// Delete dialog state and handlers
	let showDeleteDialog = false;
	let uomToDelete: UOM | null = null;
	let isDeleting = false;

	function handleDeleteClick(item: UOM) {
		uomToDelete = item;
		showDeleteDialog = true;
	}

	async function confirmDelete() {
		if (!uomToDelete || isDeleting) return;
		isDeleting = true;
		try {
			onDelete(uomToDelete.uuid);
		} finally {
			isDeleting = false;
			showDeleteDialog = false;
			uomToDelete = null;
		}
	}

	function cancelDelete() {
		if (isDeleting) return;
		showDeleteDialog = false;
		uomToDelete = null;
	}
</script>

<section class="container mx-auto px-4 py-6">
	<div class="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
		<h1 class="text-2xl font-semibold">{t('uoms_title')}</h1>
		<div class="flex items-center gap-2 flex-wrap">
			<div class="flex-1 min-w-[5ch] sm:min-w-[10ch] md:min-w-[20ch] lg:min-w-[30ch]">
				<SearchBox
					placeholder={t('press_enter_to_search')}
					value={searchQuery}
					onSearch={handleSearch}
				/>
			</div>
			{#if activeSearchQuery}
				<Button label={t('clear')} color="gray" size="sm" onClick={clearSearch} />
			{/if}
			<Button
				label={t('add_unit')}
				color="green"
				size="md"
				href={newHref()}
			/>
		</div>
	</div>

	<div class="text-muted-foreground mb-2 text-sm">
		{#if activeSearchQuery}
			{t('showing_filtered_uoms', {
				shown: filteredCount,
				total: totalCount,
				query: activeSearchQuery
			})}
		{:else}
			{t('showing_uoms', { count: totalCount })}
		{/if}
	</div>

	<!-- <div class="flex justify-end px-4 py-3">
		<Button
			label={t('add_unit')}
			color="green"
			size="md"
			href="/master/units-of-measurements/new"
		/>
	</div> -->

	<div class="table-wrapper overflow-x-auto">
		<table class="min-w-full text-left text-sm rtl:text-right">
			<thead class="table-head text-xs tracking-wide uppercase">
				<tr>
					{#each columns as col}
						<th scope="col" class="px-6 py-3 font-semibold">{col.label}</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#if items.length === 0}
					<tr>
						<td colspan={columns.length} class="px-6 py-4 text-center">{t('no_uoms_available')}</td>
					</tr>
				{:else}
					{#each items as item}
						<tr class="table-row">
							<td class="px-6 py-4 font-medium whitespace-nowrap">{item.code}</td>
							<td class="px-6 py-4">{item.name}</td>
							<td class="px-6 py-4">{formatDateTime(item.created_at)}</td>
							<td class="px-6 py-4">{formatDateTime(item.updated_at)}</td>
							<td class="px-6 py-4">
								<div class="table-actions">
									<Button
										label={t('edit')}
										color="violet"
										size="sm"
										href={editHref(item.uuid)}
									/>
									<Button
										label={t('delete')}
										color="red"
										size="sm"
										onClick={() => handleDeleteClick(item)}
									/>
								</div>
							</td>
						</tr>
					{/each}
				{/if}
			</tbody>
		</table>
	</div>

	<div class="mt-4">
		<Pagination page={currentDisplayPage} pageCount={totalPages} onChange={go} />
	</div>
</section>

<ConfirmDialog
	show={showDeleteDialog}
	title={t('delete_confirm_title_uom')}
	message={t('delete_confirm_message_uom')}
	confirmText={t('delete')}
	cancelText={t('cancel')}
	isLoading={isDeleting}
	onConfirm={confirmDelete}
	onCancel={cancelDelete}
/>

<Toaster />

<style>
	.table-wrapper {
		background: var(--color-orange-deep-600);
		color: white;
		border: 1px solid color-mix(in oklch, var(--ui-border) 70%, transparent 30%);
		border-radius: var(--radius-card);
	}

	.table-wrapper table {
		border-collapse: separate;
		border-spacing: 0;
		width: 100%;
		border-radius: var(--radius-card);
		overflow: hidden;
	}

	.table-head {
		background: var(--color-orange-deep-600);
		color: white;
	}
	.table-head th,
	.table-head div {
		color: white;
	}

	.table-row {
		background: color-mix(in oklch, var(--color-orange-deep-600) 6%, white 94%);
		color: color-mix(in oklch, var(--color-text) 90%, black 10%);
	}

	.table-row:nth-child(even) {
		background: color-mix(in oklch, var(--color-orange-deep-600) 11%, white 89%);
	}

	.table-row td {
		border-top: 1px solid color-mix(in oklch, var(--ui-border) 55%, transparent 45%);
	}

	.table-row:first-child td {
		border-top: none;
	}

	.table-actions {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		justify-content: flex-start;
	}

	:global(html[data-mode='dark']) .table-wrapper {
		background: color-mix(in oklch, var(--color-surface) 70%, transparent 30%);
		border-color: color-mix(in oklch, var(--ui-border) 80%, transparent 20%);
		box-shadow: var(--shadow-card);
	}

	:global(html[data-mode='dark']) .table-head {
		background: #111827;
		color: rgb(229 231 235);
	}

	:global(html[data-mode='dark']) .table-row,
	:global(html[data-mode='dark']) .table-row:nth-child(even) {
		background: rgba(31, 41, 55, 0.75);
		color: rgb(209 213 219);
	}

	:global(html[data-mode='dark']) .table-row td {
		border-top: 1px solid rgba(75, 85, 99, 0.45);
	}
</style>
