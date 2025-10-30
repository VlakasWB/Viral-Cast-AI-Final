<script lang="ts">
	// [ID] Konsisten: gunakan komponen Button & goto untuk navigasi
	// [EN] Consistent: reuse Button & goto for navigation
	import Button from '$lib/components/ui/Button.svelte';
	import { goto } from '$app/navigation';
</script>

<svelte:head>
	<title>404 — Page Not Found</title>
</svelte:head>

<div class="mx-auto max-w-lg text-center">
	<div
		class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-8"
	>
		<div class="mb-4 text-4xl font-bold tracking-tight">404</div>
		<h1 class="mb-2 text-xl font-semibold">Page not found</h1>
		<p class="mb-6 text-sm opacity-70">
			[ID] Maaf, halaman yang Anda cari tidak tersedia. [EN] Sorry, the page you’re looking for
			doesn’t exist.
		</p>

		<div class="flex items-center justify-center gap-3">
			<Button color="accent" label="Go to Dashboard" onClick={() => goto('/')} />
			<a
				href="/search"
				class="inline-flex items-center justify-center rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 text-sm hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)]"
			>
				Search
			</a>
		</div>
	</div>
</div>
