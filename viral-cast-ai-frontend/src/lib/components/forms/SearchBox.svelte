<script lang="ts">
	let {
		placeholder = 'Search…',
		value = '', // NEW: initial value dari parent (URL)
		onSearch
	}: { placeholder?: string; value?: string; onSearch?: (q: string) => void } = $props();

	let q = $state(value);

	// sinkron kalau parent mengganti value (mis. saat back/forward)
	$effect(() => {
		q = value ?? '';
	});

	// Auto-trigger search when input is cleared
	$effect(() => {
		if (q === '') {
			onSearch?.('');
		}
	});

	function submit(e: Event) {
		e.preventDefault();
		onSearch?.(q.trim());
	}
</script>

<form class="flex items-center gap-2" onsubmit={submit}>
	<input
		class="form-input rounded-[var(--radius-pill)] placeholder-black dark:placeholder-white"
		{placeholder}
		bind:value={q}
	/>
	<button class="rounded-[var(--radius-pill)] border border-[--ui-border] px-4 py-2" type="submit">
		Cari
	</button>
</form>
