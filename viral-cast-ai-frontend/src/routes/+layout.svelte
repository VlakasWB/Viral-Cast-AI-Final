<script lang="ts">
	import '../app.css';
	import type { Snippet } from 'svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Header from '$lib/components/Header.svelte';
	import ShoppingCart from '$lib/components/ShoppingCart.svelte';
	import { browser } from '$app/environment';
	import { initTheme } from '$lib/theme';
	import { onMount } from 'svelte';
	import { page } from '$app/state';

	let { children }: { children: Snippet } = $props();

	// state layout
	let collapsed = $state(false);
	let mobileOpen = $state(false);

	// hamburger: desktop -> collapse, mobile -> drawer
	function onHamburger(isDesktop: boolean) {
		if (isDesktop) collapsed = !collapsed;
		else mobileOpen = !mobileOpen;
	}

	// theme init
	$effect(() => initTheme());

	// close drawer saat naik ke md
	type MQLite = {
		matches: boolean;
		addEventListener: (type: 'change', listener: () => void) => void;
		removeEventListener: (type: 'change', listener: () => void) => void;
	};

	onMount(() => {
		if (!browser) return;
		const mql = (globalThis as any).matchMedia?.('(min-width: 768px)') as MQLite | undefined;
		if (!mql) return;
		const onChange = () => {
			if (mql.matches) mobileOpen = false;
		};
		mql.addEventListener('change', onChange);
		return () => mql.removeEventListener('change', onChange);
	});

	// ---- auth mode: rute /auth/* full-screen, tanpa sidebar/header
	const isAuthRoute = $derived.by(() => page.url.pathname.startsWith('/login'));
	const isPagesRoute = $derived.by(() => page.url.pathname.startsWith('/pages'));
</script>

{#if isAuthRoute || isPagesRoute}
	<!-- Full-bleed auth pages (login/register/forgot/reset/logout) -->
	<main class="min-h-dvh">
		{@render children()}
	</main>
{:else}
	<!-- App shell (sidebar + header) -->
	<div
		class="grid min-h-dvh grid-rows-[var(--header-h)_1fr] md:grid-cols-[var(--sidebar-w)_1fr]"
		style={`--header-h:64px; --sidebar-w:${collapsed ? '4.25rem' : '18rem'}`}
	>
		<!-- Sidebar (span 2 rows) -->
		<Sidebar
			class="md:col-start-1 md:row-span-2 md:row-start-1"
			{collapsed}
			open={mobileOpen}
			onClose={() => (mobileOpen = false)}
		/>

		<!-- Header -->
		<div class="md:col-start-2 md:row-start-1">
			<Header {onHamburger} />
		</div>

		<!-- Main -->
		<main class="p-4 md:col-start-2 md:row-start-2 md:p-6">
			{@render children()}
		</main>
	</div>

	<!-- Shopping Cart Component -->
	<ShoppingCart />
{/if}
