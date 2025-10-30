<script lang="ts">
	// [ID] Gunakan store $page untuk mendapat URL saat ini (SSR-safe)
	// [EN] Use $page store to get current URL (SSR-safe)
	import { page } from '$app/stores';
	import { locale, t } from '$lib/stores/i18n';

	let errorTitle = '';
	let errorDesc = '';
	let tryAgainLabel = '';
	let goHomeLabel = '';

	$: {
		const __ = $locale; // trigger re-render on locale change
		errorTitle = t('error_500_title');
		errorDesc = t('error_500_desc');
		tryAgainLabel = t('try_again');
		goHomeLabel = t('go_home');
	}
</script>

<svelte:head>
	<title>500 — {errorTitle}</title>
</svelte:head>

<div class="mx-auto max-w-lg text-center">
	<div
		class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-8"
	>
		<div class="mb-4 text-4xl font-bold tracking-tight">500</div>
		<h1 class="mb-2 text-xl font-semibold">{errorTitle}</h1>
		<p class="mb-6 text-sm opacity-70">
			{errorDesc}
		</p>

		<div class="flex items-center justify-center gap-3">
			<!-- [ID] “Try Again” = reload ke URL saat ini tanpa window/location -->
			<!-- [EN] “Try Again” = reload current URL without window/location -->
			<a
				href={$page.url.pathname + $page.url.search}
				class="inline-flex items-center justify-center rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 text-sm hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)]"
			>
				{tryAgainLabel}
			</a>

			<a
				href="/"
				class="inline-flex items-center justify-center rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 text-sm hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)]"
			>
				{goHomeLabel}
			</a>
		</div>
	</div>
</div>
