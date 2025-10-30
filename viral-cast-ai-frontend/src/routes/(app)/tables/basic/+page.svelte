<!-- <script lang="ts">
  import BasicTable from '$lib/components/tables/BasicTable.svelte';
  import Pagination from '$lib/components/Pagination.svelte';

  // [ID] Demo data (nanti diganti fetch API / store).
  // [EN] Demo data (later replace with API/store).
  type Row = { id: number; name: string; price: number };
  const all: Row[] = Array.from({ length: 23 }, (_, i) => ({ id: i + 1, name: `Item ${i+1}`, price: (i+1) * 1000 }));

  let page = $state(1);
  const size = 5;
  $derived paged = all.slice((page - 1) * size, page * size);
  $derived pageCount = Math.max(1, Math.ceil(all.length / size));

  function onChange(p: number) { page = p; }
</script>

<section class="space-y-4">
	<h1 class="text-2xl font-semibold">Basic Table</h1>
	<BasicTable columns={['id', 'name', 'price']} rows={paged} />
	<div class="flex justify-end">
		<Pagination {page} {pageCount} {onChange} />
	</div>
</section> -->
