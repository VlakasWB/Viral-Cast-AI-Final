<script lang="ts">
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	let { children }: { children: Snippet } = $props();

	const isAuthPages = $derived.by(() => page.url.pathname.startsWith('/login'));
</script>

{#if isAuthPages}
	{@render children()}
{:else}
	<section class="grid min-h-dvh place-items-center bg-[var(--app-bg)] px-4 py-10">
		<div class="w-full max-w-md">
			{@render children()}
		</div>
	</section>
{/if}
