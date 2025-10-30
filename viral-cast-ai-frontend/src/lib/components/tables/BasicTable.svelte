<script lang="ts">
	// [ID] Tabel sederhana: kolom header + array data.
	// [EN] Simple table: column headers + array of data.
	type Row = Record<string, string | number | boolean>;
	let { columns = [] as string[], rows = [] as Row[] } = $props();
</script>

<div
	class="overflow-x-auto rounded-[var(--radius-card)] border bg-white/60 backdrop-blur-md dark:bg-white/10"
	style="border-color: var(--ui-border);"
>
	<table class="min-w-full text-sm">
		<thead class="bg-white/60 dark:bg-white/10">
			<tr>
				{#each columns as col}
					<th class="px-4 py-2 text-left font-semibold">{col}</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{#each rows as row}
				<tr class="odd:bg-white/40 dark:odd:bg-white/5">
					{#each columns as col}
						<td class="px-4 py-2">{String(row[col] ?? '')}</td>
					{/each}
				</tr>
			{/each}
			{#if rows.length === 0}
				<tr><td class="px-4 py-6 opacity-60" colspan={columns.length}>No data</td></tr>
			{/if}
		</tbody>
	</table>
</div>
