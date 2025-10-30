<script lang="ts">
	// [ID] Toggle mode gelap/terang, aman SSR/CSR.
	// [EN] Dark/Light toggle, SSR/CSR-safe.
	import { browser } from '$app/environment';
	import { t } from '$lib/stores/i18n';

	let mode = $state<'light' | 'dark'>('light');

	$effect(() => {
		if (!browser) return;
		const d = (globalThis as any).document?.documentElement;
		const saved = (globalThis as any).localStorage?.getItem('mode') as 'light' | 'dark' | null;
		mode = saved ?? (d?.dataset?.mode as 'light' | 'dark') ?? 'light';
		if (d) {
			d.dataset.mode = mode;
			d.classList.toggle('dark', mode === 'dark'); // [CHANGED] fallback untuk util dark:*
		}
	});

	function toggle() {
		if (!browser) return;
		mode = mode === 'light' ? 'dark' : 'light';
		const d = (globalThis as any).document?.documentElement;
		if (d) {
			d.dataset.mode = mode;
			d.classList.toggle('dark', mode === 'dark'); // [CHANGED]
		}
		(globalThis as any).localStorage?.setItem('mode', mode);
	}
</script>

<button
	type="button"
	onclick={toggle}
	class="shadow-header inline-flex items-center gap-2 rounded-full border border-[var(--ui-border)] bg-[var(--color-surface)] px-2.5 py-1.5 transition hover:opacity-100 focus-visible:ring-2 focus-visible:ring-[--nav-active-ring] focus-visible:outline-none"
	aria-label="Toggle dark mode"
>
	{mode === 'light' ? `🌞 ${t('theme_light')}` : `🌙 ${t('theme_dark')}`}
</button>
