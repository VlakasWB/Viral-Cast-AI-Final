<script lang="ts">
  let {
    page = 1,
    pageCount = 1,
    total = 0,
    start = 0,
    end = 0,
    size = 10,
    sizes = [3, 5, 10, 25, 50],
    onChangePage,
    onChangeSize
  }: {
    page?: number;
    pageCount?: number;
    total?: number;
    start?: number;
    end?: number;
    size?: number;
    sizes?: number[];
    onChangePage?: (p: number) => void;
    onChangeSize?: (s: number) => void;
  } = $props();

  function go(p: number) {
    if (p < 1 || p > pageCount) return;
    page = p;
    onChangePage?.(page);
  }

  function changeSize(val: number) {
    size = val;
    onChangeSize?.(size);
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

    add(1);
    const startN = Math.max(2, page - 1);
    const endN = Math.min(pageCount - 1, page + 1);
    for (let i = startN; i <= endN; i++) add(i);
    if (page <= 3) add(4);
    if (page >= pageCount - 2) add(pageCount - 3);

    pages.sort((a, b) => (a === '…' || b === '…' ? 0 : (a as number) - (b as number)));

    const final: (number | '…')[] = [];
    for (let i = 0; i < pages.length; i++) {
      const curr = pages[i] as number;
      const prev = final[final.length - 1] as number | undefined;
      if (prev && curr - prev > 1) final.push('…');
      final.push(curr);
    }
    if (final[final.length - 1] !== pageCount) final.push('…', pageCount);
    return final;
  }
</script>

<div class="flex flex-col items-center justify-between gap-3 px-2 py-2 sm:flex-row">
  <div class="text-sm text-gray-700 dark:text-gray-300">
    {#if total > 0 && start > 0 && end > 0}
      Showing {start} to {end} of {total} Entries
    {:else}
      Showing 0 of {total} Entries
    {/if}
  </div>

  <div class="flex items-center gap-3">
    <label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
      <span>Show</span>
      <select
        class="form-input w-[110px]"
        bind:value={size}
        onchange={(e) => changeSize(Number((e.target as HTMLSelectElement).value))}
      >
        {#each sizes as opt}
          <option value={opt}>{opt}</option>
        {/each}
      </select>
      <span>/ page</span>
    </label>

    <nav class="inline-flex items-center gap-1">
      <button
        class="rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-3 py-1 disabled:opacity-50"
        disabled={page <= 1}
        onclick={() => go(page - 1)}
        aria-label="Previous page"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" class="inline">
          <path fill="currentColor" d="M15 18l-6-6 6-6" />
        </svg>
      </button>

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
        onclick={() => go(page + 1)}
        aria-label="Next page"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" class="inline">
          <path fill="currentColor" d="M9 6l6 6-6 6" />
        </svg>
      </button>
    </nav>
  </div>
</div>