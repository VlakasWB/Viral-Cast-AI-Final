<script lang="ts">
	// ✅ gunakan page dari $app/state (runes-aware)
	import { page } from '$app/state';
  import { t } from '$lib/stores/i18n';
  import { goto } from '$app/navigation';

	let {
		collapsed = false,
		open = false,
		onClose,
		class: rootClass = ''
	}: {
		collapsed?: boolean;
		open?: boolean;
		onClose?: () => void;
		class?: string;
	} = $props();

  const close = () => onClose?.();

  function navigate(href: string) {
    // Tutup sidebar (mobile) lalu navigasi via SvelteKit
    close();
    goto(href);
  }

	// Ambil path reaktif (tanpa subscribe manual) dengan null check
	const pathname = $derived.by(() => page?.url?.pathname ?? '/') as string;

	function isActive(href: string) {
		if (href === '/') return pathname === '/';
		return pathname === href || pathname.startsWith(href + '/');
	}

	// util kelas item nav
	const baseItem =
		'group flex items-center gap-3 rounded-xl px-3 py-2.5 border transition-colors outline-none ' +
		'border-transparent text-[--color-text] hover:bg-black/5 dark:hover:bg-white/5';

	// state aktif → gradien mengikuti theme (lihat app.css: --nav-active-bg)
	const activeMore =
		'data-[active=true]:nav-active-bg data-[active=true]:text-[--nav-active-text] ' +
		'data-[active=true]:ring-1 data-[active=true]:ring-[--nav-active-ring] ' +
		'data-[active=true]:border-[--nav-active-ring]';

	// --- Dropdown Auth ---
	const isAuthMatch = $derived.by(() => {
		if (!pathname) return false;
		return pathname === '/auth' || pathname.startsWith('/auth/');
	});

	// Preferensi user: null=auto (ikut route), expanded/collapsed=override
	let authUser: 'expanded' | 'collapsed' | null = $state(null);

	// Tampil jika: tidak collapsed & (override user atau match route)
	const showAuth = $derived.by(() => {
		if (collapsed) return false;
		if (authUser === 'expanded') return true;
		if (authUser === 'collapsed') return false;
		return isAuthMatch;
	});

	function toggleAuth() {
		authUser = showAuth ? 'collapsed' : 'expanded';
	}

	// Reset override saat keluar dari namespace /auth/*
	$effect(() => {
		if (!isAuthMatch) authUser = null;
	});

	// Handler keyboard khusus caret Auth (tanpa .key modifier)
	function onAuthCaretKeydown(e: Event & { key?: string }) {
		const key = e.key ?? (e as any).key;
		if (key === 'Enter' || key === ' ' || key === 'Spacebar') {
			e.preventDefault();
			e.stopPropagation();
			toggleAuth();
		}
	}

	const isAIMatch = $derived.by(() => {
		if (!pathname) return false;
		return pathname === '/ai' || pathname.startsWith('/ai/');
	});

	let aiUser: 'expanded' | 'collapsed' | null = $state(null);

	const showAI = $derived.by(() => {
		if (collapsed) return false;
		if (aiUser === 'expanded') return true;
		if (aiUser === 'collapsed') return false;
		return isAIMatch;
	});

	function toggleAI() {
		aiUser = showAI ? 'collapsed' : 'expanded';
	}

	$effect(() => {
		if (!isAIMatch) aiUser = null;
	});

	function onAICaretKeydown(e: Event & { key?: string }) {
		const key = e.key ?? (e as any).key;
		if (key === 'Enter' || key === ' ' || key === 'Spacebar') {
			e.preventDefault();
			e.stopPropagation();
			toggleAI();
		}
	}
</script>

{#if open}
	<!-- backdrop mobile -->
	<div class="fixed inset-0 z-40 bg-black/40 md:hidden" onclick={close} aria-hidden="true"></div>
{/if}

<aside
    class={`fixed top-16 bottom-0 left-0 z-50 w-[16rem]
          transform border-r border-[var(--ui-border)]
          bg-[var(--color-surface)] shadow-[var(--shadow-sidebar)] transition-transform
          md:static md:top-0 md:z-auto md:w-auto md:translate-x-0
          ${open ? 'translate-x-0' : '-translate-x-full md:translate-x-0'}
          ${collapsed ? 'md:w-[3.75rem]' : 'md:w-[16rem]'}
          ${rootClass}`}
    aria-label="Sidebar"
>
	<div class="flex h-full flex-col overflow-hidden">
		<!-- Brand -->
		<div class="flex items-center gap-3 px-4 pt-4">
			<div
				class="grid h-8 w-8 place-items-center rounded-lg border border-[var(--ui-border)] bg-white/70 dark:bg-white/10"
			>
				<span class="text-xs font-bold">VC</span>
			</div>
			<div class={`min-w-0 ${collapsed ? 'md:hidden' : ''}`}>
				<div class="truncate text-base font-semibold">Viral Cast AI</div>
			</div>
			<button
				class="ml-auto rounded-full border border-[var(--ui-border)] p-2 md:hidden"
				onclick={close}
				aria-label="Close"
			>
				<svg
					viewBox="0 0 24 24"
					width="18"
					height="18"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path d="M6 6l12 12M18 6 6 18" />
				</svg>
			</button>
		</div>

		<!-- Scrollable Navigation Content -->
		<div class="flex-1 overflow-y-auto pb-4">
			<!-- Divider MENU -->
			<div class={`px-4 pt-3 ${collapsed ? 'md:hidden' : ''}`}>
				<div class="text-xs tracking-wide uppercase opacity-60">{t('menu')}</div>
			</div>

			<!-- NAV group 1 -->
			<nav class="mt-2 space-y-1 px-2">
				<!-- Dashboard -->
				<a
					href="/"
					data-active={isActive('/')}
				class={`${baseItem} ${activeMore}`}
				title={t('nav_dashboard')}
				onclick={close}
			>
					<svg
						viewBox="0 0 24 24"
						class="h-5 w-5 opacity-80"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
					>
						<path d="M3 10.5 12 3l9 7.5V21a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1v-10.5z" />
					</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>{t('nav_dashboard')}</span>
			</a>

				<!-- Orders -->
				<!-- <a
				href="/orders"
				data-active={isActive('/orders')}
				class={`${baseItem} ${activeMore}`}
				title="Orders"
				onclick={close}
			>
				<svg
					viewBox="0 0 24 24"
					class="h-5 w-5 opacity-80"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
				>
					<path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
				</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Orders</span>
			</a> -->

				<!-- Products -->
				<a
					href="/products"
					data-active={isActive('/products')}
				class={`${baseItem} ${activeMore}`}
				title={t('nav_products')}
				onclick={close}
			>
					<svg
						viewBox="0 0 24 24"
						class="h-5 w-5 opacity-80"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
					>
						<path
							d="M3 3h2l.4 2M7 13h10l4-8H5.4m0 0L7 13m0 0l-1.5 6M7 13l-1.5-6m0 0L4 5M7 13h10m0 0l1.5 6M17 13l1.5-6"
						/>
					</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>{t('nav_products')}</span>
			</a>

				<!-- Weather -->
				<a
					href="/weather"
					data-active={isActive('/weather')}
				class={`${baseItem} ${activeMore}`}
				title={t('weather')}
				onclick={close}
			>
					<svg
						viewBox="0 0 24 24"
						class="h-5 w-5 opacity-80"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
					>
						<path d="M6 19h10a4 4 0 0 0 0-8 6 6 0 0 0-11.5 2 3.5 3.5 0 0 0 1.5 6z" />
					</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>{t('weather')}</span>
			</a>

				<div class="h-3"></div>
			</nav>

			<!-- Divider Products -->
			<div class={`px-4 pt-3 ${collapsed ? 'md:hidden' : ''}`}>
				<div class="text-xs tracking-wide uppercase opacity-60">{t('Products')}</div>
			</div>

			<!-- NAV Products -->
			<nav class="mt-2 space-y-1 px-2">
				<!-- Master Products -->
				<a
					href="/master/products"
					data-active={isActive('/master/products')}
					class={`${baseItem} ${activeMore}`}
					title={t('nav_products')}
					onclick={close}
				>
					<svg
						viewBox="0 0 24 24"
						class="h-5 w-5 opacity-80"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
					>
						<path d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z" />
						<path
							d="M4 12h2m12 0h2M12 4v2m0 12v2M6 6l1.5 1.5M16.5 16.5 18 18M6 18l1.5-1.5M16.5 7.5 18 6"
						/>
					</svg>
					<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>{t('nav_products')}</span>
				</a>

				<!-- Master Categories -->
                <a
                    href="/master/categories"
                    data-active={isActive('/master/categories')}
                    class={`${baseItem} ${activeMore}`}
                    title={t('nav_categories')}
                    onclick={() => navigate('/master/categories')}
                >
					<svg
						viewBox="0 0 24 24"
						class="h-5 w-5 opacity-80"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
					>
						<path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2Z" />
						<path d="M8 5v4" />
						<path d="M16 5v4" />
						<path d="M3 9h18" />
					</svg>
					<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>{t('nav_categories')}</span>
				</a>

				<!-- Master Recipes -->
				<a
					href="/master/recipes"
					data-active={isActive('/master/recipes')}
					class={`${baseItem} ${activeMore}`}
					title={t('nav_recipes')}
					onclick={close}
				>
					<svg
						viewBox="0 0 24 24"
						class="h-5 w-5 opacity-80"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
					>
						<path
							d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
						/>
					</svg>
					<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>{t('nav_recipes')}</span>
				</a>
				<div class="h-3"></div>
			</nav>

			<!-- Divider Ingredients -->
			<div class={`px-4 pt-3 ${collapsed ? 'md:hidden' : ''}`}>
				<div class="text-xs tracking-wide uppercase opacity-60">{t('Ingredients')}</div>
			</div>

			<!-- NAV Ingredients -->
			<nav class="mt-2 space-y-1 px-2">
				<!-- Ingredients Stocks -->
				<a
					href="/master/ingredient-stocks"
					data-active={isActive('/master/ingredient-stocks')}
					class={`${baseItem} ${activeMore}`}
					title={t('nav_ingredient_stocks')}
					onclick={close}
				>
					<svg
						viewBox="0 0 24 24"
						class="h-5 w-5 opacity-80"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
					>
						<rect x="3" y="4" width="18" height="16" rx="2" />
						<path d="M3 9h18M8 9v11M16 9v11" />
					</svg>
					<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>{t('Ingredient Stocks')}</span>
				</a>

				<!-- Ingredients Stock Moves -->
				<a
					href="/master/ingredient-stock-moves"
					data-active={isActive('/master/ingredient-stock-moves')}
					class={`${baseItem} ${activeMore}`}
					title={t('nav_ingredient_stock_moves')}
					onclick={close}
				>
					<svg
						viewBox="0 0 24 24"
						class="h-5 w-5 opacity-80"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
					>
						<rect x="3" y="4" width="18" height="16" rx="2" />
						<path d="M3 9h18M8 9v11M16 9v11" />
					</svg>
					<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>{t('Ingredient Stock Moves')}</span>
				</a>

				<!-- Ingredients Catalog -->
				<a
					href="/master/ingredient-catalog"
					data-active={isActive('/master/ingredient-catalog')}
					class={`${baseItem} ${activeMore}`}
					title={t('nav_ingredient_catalog')}
					onclick={close}
				>
					<svg
						viewBox="0 0 24 24"
						class="h-5 w-5 opacity-80"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
					>
						<rect x="3" y="4" width="18" height="16" rx="2" />
						<path d="M3 9h18M8 9v11M16 9v11" />
					</svg>
					<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>{t('Ingredient Catalog')}</span>
				</a>

				<!-- Units of Measurements Products -->
				<a
					href="/master/units-of-measurements"
					data-active={isActive('/master/units-of-measurements')}
					class={`${baseItem} ${activeMore}`}
					title={t('nav_uoms')}
					onclick={close}
				>
					<svg
						viewBox="0 0 24 24"
						class="h-5 w-5 opacity-80"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
					>
						<rect x="3" y="4" width="18" height="16" rx="2" />
						<path d="M3 9h18M8 9v11M16 9v11" />
					</svg>
					<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>{t('nav_uoms')}</span>
				</a>

				<div class="h-3"></div>
			</nav>

			<!-- Divider: AI -->
			<div class={`px-4 pt-5 ${collapsed ? 'md:hidden' : ''}`}>
				<div class="text-xs tracking-wide uppercase opacity-60">AI</div>
			</div>

			<!-- NAV group: AI -->
			<nav class="mt-2 space-y-1 px-2">
				<!-- Parent: AI -->
				<a
					href="/ai"
					data-active={isActive('/ai')}
					class={`${baseItem} ${activeMore}`}
					title="AI"
					aria-expanded={showAI}
					aria-controls="submenu-ai"
					onclick={(e) => {
						if (isAIMatch) {
							e.preventDefault();
							e.stopPropagation();
							toggleAI();
							return;
						}
						aiUser = 'expanded';
					}}
				>
					<!-- ikon AI (sparkle) -->
					<svg
						viewBox="0 0 24 24"
						class="h-5 w-5 opacity-80"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
					>
						<path
							d="M12 3l1.5 3.5L17 8l-3.5 1.5L12 13l-1.5-3.5L7 8l3.5-1.5L12 3zM5 16l.8 1.8L8 18l-1.8.8L5 21l-.8-1.8L2 18l2.2-.2L5 16zm14 0l.8 1.8L22 18l-1.8.8L19 21l-.8-1.8L16 18l2.2-.2L19 16z"
						/>
					</svg>

					<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>AI</span>

					<!-- caret -->
					<span
						role="button"
						tabindex="0"
						class={`ml-auto rounded p-1 opacity-70 transition-transform duration-200 ${collapsed ? 'md:hidden' : ''} ${showAI ? 'rotate-90' : ''}`}
						aria-label="Toggle AI menu"
						aria-expanded={showAI}
						onclick={(e) => {
							e.preventDefault();
							e.stopPropagation();
							toggleAI();
						}}
						onkeydown={onAICaretKeydown}
					>
						<svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
							<path fill="currentColor" d="M9 6l6 6-6 6" />
						</svg>
					</span>
				</a>

				{#if showAI}
					<ul id="submenu-ai" class="mt-1 space-y-1">
						<li>
							<a
								href="/ai"
								data-active={isActive('/ai')}
								class={`${baseItem} ${activeMore} pl-9`}
								title="AI Chat"
								onclick={close}
							>
								<svg
									viewBox="0 0 24 24"
									class="h-4 w-4 opacity-80"
									fill="none"
									stroke="currentColor"
									stroke-width="1.8"
								>
									<path
										d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
									/>
								</svg>
								<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>AI Chat</span>
							</a>
						</li>
					</ul>
				{/if}
			</nav>
		</div>
		<!-- End Scrollable Navigation Content -->
	</div>
</aside>
