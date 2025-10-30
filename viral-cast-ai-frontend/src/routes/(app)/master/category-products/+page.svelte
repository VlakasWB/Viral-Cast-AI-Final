<script lang="ts">
	import Pagination from '$lib/components/Pagination.svelte';
	import SearchBox from '$lib/components/SearchBox.svelte';
	import type { Product } from '$lib/types';
	import { goto, invalidateAll } from '$app/navigation';
	import Button from '$lib/components/ui/Button.svelte';

	let {
		data
	}: {
		data: {
			items: Product[];
			total: number;
			page: number;
			pageCount: number;
			q: string;
			size: number;
		};
	} = $props();

	// simpan q lokal agar konsisten dengan URL setelah navigasi
	let q = $state(data.q);
	$effect(() => {
		// sinkron saat data berubah akibat navigasi (next/prev page, search baru, dll)
		q = data.q;
	});

	// Pagination -> update ?page=...
	function go(p: number) {
		const params = new URLSearchParams();
		if (q) params.set('q', q);
		params.set('page', String(p));
		goto(`/products?${params.toString()}`);
	}

	// Delete -> action + refresh data
	async function handleDelete(id: string) {
		if (!(globalThis as any).confirm?.('Delete this product?')) return;
		const fd = new FormData();
		fd.set('id', id);
		await fetch('?/delete', { method: 'POST', body: fd });
		await invalidateAll(); // refresh data halaman
	}

	// Search -> set ?q=...&page=1
	function handleSearch(nextQ: string) {
		q = nextQ;
		const params = new URLSearchParams();
		if (q) params.set('q', q);
		params.set('page', '1'); // reset ke halaman pertama
		goto(`/products?${params.toString()}`);
	}

	const columns = [
		{ key: 'name', label: 'Name' },
		{ key: 'price', label: 'Price' },
		{ key: 'stock', label: 'Stock' },
		{ key: 'active', label: 'Active' },
		{ key: 'actions', label: 'Actions' }
	] as const;
</script>

<section class="space-y-4">
	<h1 class="text-xl font-semibold">Products</h1>
	<!-- Header bar: title + search + add -->
	<div class="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
		<div class="flex items-center gap-2">
			<SearchBox onSearch={handleSearch} />
		</div>
		<Button label="Add Product" color="emerald" href="/master/products/new" />
	</div>

	<!-- Table -->
	<div
		class="overflow-x-auto rounded-[var(--radius-card)] border bg-white/60 backdrop-blur-md dark:bg-white/10" style="border-color: var(--ui-border);"
	>
    <table class="min-w-full text-sm">
            <thead class="bg-gray-50 dark:bg-gray-900">
                <tr>
                    {#each columns as c}
                        <th class="px-4 py-2 text-left font-semibold">{c.label}</th>
                    {/each}
                </tr>
            </thead>
            <tbody>
                {#each data.items as row}
                    <tr class="odd:bg-gray-50 dark:odd:bg-gray-800">
                        <td class="px-4 py-2">{row.name}</td>
                        <td class="px-4 py-2">{row.price}</td>
						<td class="px-4 py-2">{row.stock}</td>
						<td class="px-4 py-2">{row.active ? 'Yes' : 'No'}</td>
						<td class="px-4 py-2">
							<div class="flex items-center gap-2">
								<Button label="Edit" color="violet" href={`/products/${row.id}/edit`} />
								<Button label="Delete" color="red" onClick={() => handleDelete(row.id)} />
							</div>
						</td>
					</tr>
				{/each}

				{#if data.items.length === 0}
					<tr>
						<td class="px-4 py-6 opacity-60" colspan={columns.length}>No data</td>
					</tr>
				{/if}
			</tbody>
		</table>
	</div>

	<!-- Pagination -->
	<div class="flex justify-end">
		<Pagination page={data.page} pageCount={data.pageCount} onChange={go} />
	</div>
</section>
