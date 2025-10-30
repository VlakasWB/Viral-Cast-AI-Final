<script lang="ts">
	// [ID] Menghindari Web API: cukup gunakan anchor ke URL saat ini.
	// [EN] Avoid Web APIs: just use an anchor to the current URL.
	import { page } from '$app/stores';
</script>

<svelte:head>
	<title>503 — Service Unavailable</title>
</svelte:head>

<div class="mx-auto max-w-lg text-center">
	<div
		class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-8"
	>
		<div class="mb-4 text-4xl font-bold tracking-tight">503</div>
		<h1 class="mb-2 text-xl font-semibold">Service unavailable</h1>
		<p class="mb-6 text-sm opacity-70">
			[ID] Layanan sementara tidak tersedia (maintenance atau overload). [EN] The service is
			temporarily unavailable (maintenance or overload).
		</p>

		<div class="flex items-center justify-center gap-3">
			<a
				href={$page.url.pathname + $page.url.search}
				class="inline-flex items-center justify-center rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 text-sm hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)]"
			>
				Refresh
			</a>

			<a
				href="/status"
				class="inline-flex items-center justify-center rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 text-sm hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)]"
			>
				Status Page
			</a>
		</div>
	</div>
</div>
