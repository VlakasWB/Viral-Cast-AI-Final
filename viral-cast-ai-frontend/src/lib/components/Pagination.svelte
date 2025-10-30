<script lang="ts">
	// [ID] Pagination terkontrol + callback onChange(page).
	// [EN] Controlled pagination + onChange(page) callback.
	let {
		page = 1,
		pageCount = 1,
		onChange
	}: { page?: number; pageCount?: number; onChange?: (p: number) => void } = $props();

	function go(p: number) {
		if (p < 1 || p > pageCount) return;
		page = p;
		onChange?.(page);
	}

	function visiblePages(): (number | '…')[] {
		const pages: (number | '…')[] = [];
		if (pageCount <= 7) {
			for (let i = 1; i <= pageCount; i++) pages.push(i);
			return pages;
		}

		const add = (p: number) => {
			if (!pages.includes(p)) pages.push(p);
		};

		// Always show first and last
		add(1);
		// Left range around current
		const start = Math.max(2, page - 1);
		const end = Math.min(pageCount - 1, page + 1);
		for (let i = start; i <= end; i++) add(i);

		// Add neighbors farther when near edges
		if (page <= 3) add(4);
		if (page >= pageCount - 2) add(pageCount - 3);

		// Sort unique pages
		pages.sort((a, b) => (a === '…' || b === '…' ? 0 : (a as number) - (b as number)));

		// Insert ellipses
		const final: (number | '…')[] = [];
		for (let i = 0; i < pages.length; i++) {
			const curr = pages[i] as number;
			const prev = final[final.length - 1] as number | undefined;
			if (prev && curr - prev > 1) final.push('…');
			final.push(curr);
		}

		// Ensure last
		if (final[final.length - 1] !== pageCount) final.push('…', pageCount);

		// Ensure first
		if (final[0] !== 1) final.unshift(1, '…');

		return final;
	}
</script>

<nav class="flex items-center gap-2">
    <button
        class="rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-3 py-1 disabled:opacity-50"
        disabled={page <= 1}
        onclick={() => go(page - 1)}>Prev</button
    >

    <span class="px-2 text-sm opacity-80">Page {page} / {pageCount}</span>

    <!-- Numbered pages -->
    <div class="flex items-center gap-1">
        {#each visiblePages() as p}
            {#if p === '…'}
                <span class="px-2 text-gray-500">…</span>
            {:else}
                <button
                    class="rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-3 py-1 text-sm"
                    onclick={() => go(p as number)}
                    aria-current={p === page ? 'page' : undefined}
                >{p}</button>
            {/if}
        {/each}
    </div>

    <button
        class="rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-3 py-1 disabled:opacity-50"
        disabled={page >= pageCount}
        onclick={() => go(page + 1)}>Next</button
    >
</nav>
