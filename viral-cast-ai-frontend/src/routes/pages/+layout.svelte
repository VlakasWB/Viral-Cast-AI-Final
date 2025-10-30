<script lang="ts">
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	let { children }: { children: Snippet } = $props();

	// penting: pakai trailing slash biar tidak match "pages/authors"
	const isAuthPages = $derived.by(() => page.url.pathname.startsWith('/pages/auth/'));
</script>

{#if isAuthPages}
	<!-- /pages/auth/*: TIDAK ada container sempit -->
	{@render children()}
{:else}
	<!-- Halaman /pages lain masih pakai kartu tengah -->
	<section class="grid min-h-dvh place-items-center bg-[var(--app-bg)] px-4 py-10">
		<div class="w-full max-w-md">
			{@render children()}
		</div>
	</section>
{/if}
