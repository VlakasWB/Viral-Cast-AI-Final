<script lang="ts">
	import ThemePicker from '$lib/components/ThemePicker.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import LanguagePicker from '$lib/components/LanguagePicker.svelte';
	import UserMenu from '$lib/components/layout/UserMenu.svelte';
	import { cartSummary, cartActions } from '$lib/stores/cartStore';
	import { t } from '$lib/stores/i18n';

	// Props: gabungkan sekali pemanggilan $props
	let {
		onHamburger,
		class: className = '',
		name = 'Admin',
		email = 'viralcastai@gmail.com',
		avatarUrl = '/images/user.png'
	}: {
		onHamburger?: (isDesktop: boolean) => void;
		class?: string;
		name?: string;
		email?: string;
		avatarUrl?: string;
	} = $props();

	// Klik hamburger → cek breakpoint md (>=768px)
	function handleHamburger() {
		const mql: { matches: boolean } | undefined = (globalThis as any)?.matchMedia?.(
			'(min-width: 768px)'
		);
		const isDesktop = !!mql?.matches;
		onHamburger?.(isDesktop);
	}
</script>

<header
	class={`sticky top-0 z-40 border-b border-[var(--ui-border)]
          bg-[var(--color-surface)]/80 shadow-[var(--shadow-header)]
          backdrop-blur-md ${className}`}
>
	<div class="mx-auto flex h-14 items-center justify-between px-4">
		<!-- LEFT: hamburger + title -->
		<div class="flex items-center gap-3">
			<button
				type="button"
				onclick={handleHamburger}
				aria-label="Toggle sidebar"

			>
				<svg
					width="18"
					height="18"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path d="M4 6h16M4 12h16M4 18h16" />
				</svg>
			</button>

			<span class="text-[15.5px] font-semibold tracking-wide">{t('menu')}</span>
		</div>

		<!-- RIGHT: cart + theme controls + user menu -->
		<div class="flex items-center gap-2">
			<!-- Shopping Cart Icon -->
			<button
				type="button"
				onclick={cartActions.toggleCart}
				aria-label="Shopping cart"
				class="shadow-header relative inline-flex h-9 w-9 items-center justify-center rounded-full border border-[var(--ui-border)] bg-[var(--color-surface)] transition hover:opacity-100 focus-visible:ring-2 focus-visible:ring-[--nav-active-ring] focus-visible:outline-none"
			>
				<svg
					width="18"
					height="18"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path
						d="M3 3h2l.4 2M7 13h10l4-8H5.4m0 0L7 13m0 0l-1.5 6M7 13l-1.5-6m0 0L4 5M7 13h10m0 0l1.5 6M17 13l1.5-6"
					/>
				</svg>

				<!-- Cart Item Count Badge -->
				{#if $cartSummary.itemCount > 0}
					<span
					>
						{$cartSummary.itemCount > 99 ? '99+' : $cartSummary.itemCount}
					</span>
				{/if}
			</button>

			<ThemePicker />
			<ThemeToggle />
			<LanguagePicker />

			<div class="mx-2 h-6 w-px bg-[var(--ui-border)]" aria-hidden="true"></div>

			<!-- Avatar dropdown -->
			<UserMenu
					{name}
					{email}
					{avatarUrl}
				/>
		</div>
	</div>
</header>
