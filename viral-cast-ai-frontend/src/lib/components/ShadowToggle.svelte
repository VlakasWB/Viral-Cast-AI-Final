<script lang="ts">
	import { browser } from '$app/environment';
	import { setShadow, toggleShadow } from '$lib/theme';

	let on = $state(false);

	$effect(() => {
		if (!browser) return;
		const d = (globalThis as any).document?.documentElement;
		const saved = (globalThis as any).localStorage?.getItem('shadow') as 'on' | 'off' | null;
		on = (saved ?? (d?.dataset?.shadow as 'on' | 'off') ?? 'off') === 'on';
		setShadow(on);
	});

	function toggle() {
		if (!browser) return;
		toggleShadow();
		const d = (globalThis as any).document?.documentElement;
		on = (d?.dataset?.shadow as 'on' | 'off') === 'on';
	}
</script>

<button
	type="button"
	onclick={toggle}
	class="rounded-[var(--radius-pill)] border bg-[--surface] px-3 py-1.5 hover:bg-white/55 dark:hover:bg-white/10"
>
	{on ? 'Shadow: On' : 'Shadow: Off'}
</button>
