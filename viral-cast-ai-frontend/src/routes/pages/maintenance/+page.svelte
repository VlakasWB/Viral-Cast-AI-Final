<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	// import { browser } from '$app/environment'; // tidak diperlukan lagi
</script>

<svelte:head>
	<title>Maintenance</title>
</svelte:head>

<div class="mx-auto max-w-xl text-center">
	<div
		class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-8"
	>
		<!-- ikon & copy seperti sebelumnya -->

		<div class="flex items-center justify-center gap-3">
			<a
				href="/status"
				class="inline-flex items-center justify-center rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 text-sm hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)]"
			>
				Check Status
			</a>

			<!-- Ganti Button onClick -> anchor mailto -->
			<a
				href="mailto:support@example.com"
				class="inline-flex items-center justify-center rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 text-sm hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)]"
			>
				Contact Support
			</a>
		</div>
	</div>
</div>
