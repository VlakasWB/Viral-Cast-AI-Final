<script lang="ts">
	// [ID] Picker brand theme dinonaktifkan, tema default diatur ke slate.
	// [EN] Brand theme picker disabled, default theme set to slate.
	import { browser } from '$app/environment';

	// Tema default diatur ke slate (netral)
	let current = $state<string>('slate');

	$effect(() => {
		if (!browser) return;
		// [REMOVED] setTheme dihapus; tidak ada aksi tema.
	});
</script>

<!-- Komponen dinonaktifkan dan disembunyikan -->
<!-- <div class="relative">
	<button
		type="button"
		class="shadow-header inline-flex items-center gap-2 rounded-full border border-[var(--ui-border)] bg-[var(--color-surface)] px-2.5 py-1.5 transition hover:opacity-100 focus-visible:ring-2 focus-visible:ring-[--nav-active-ring] focus-visible:outline-none"
		aria-haspopup="listbox"
		aria-expanded={open}
		onclick={() => (open = !open)}
	>
		{current.charAt(0).toUpperCase() + current.slice(1)}
	</button>

	{#if open}
		<div
			class="absolute right-0 z-10 mt-2 min-w-40 rounded-xl border border-[--glass-border] bg-[--surface] p-2 shadow-[--glass-shadow] backdrop-blur-md"
		>
			{#each themes as t}
				<button
					type="button"
					class="w-full rounded-xl px-3 py-2 text-left hover:bg-[color-mix(in_oklch,var(--color-surface),black_4%)] focus-visible:ring-2 focus-visible:ring-[--nav-active-ring] focus-visible:outline-none"
					onclick={() => choose(t)}
				>
					{t === current ? '✓ ' : ''}{t}
				</button>
			{/each}
		</div>
	{/if}
</div> -->
