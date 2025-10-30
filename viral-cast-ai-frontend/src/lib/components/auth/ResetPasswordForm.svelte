<script lang="ts">
	// Svelte 5 runes, path reaktif dari $app/state
	import { page } from '$app/state';

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
	const pathname = $derived.by(() => page.url.pathname);

	function isActive(href: string) {
		if (href === '/') return pathname === '/';
		return pathname === href || pathname.startsWith(href + '/');
	}

	// util kelas nav
	const baseItem =
		'group flex items-center gap-3 rounded-xl px-3 py-2.5 border transition-colors outline-none ' +
		'border-transparent text-[--color-text] hover:bg-black/5 dark:hover:bg-white/5';

	// state aktif → gradien & ring ikut theme (dari app.css)
	const activeMore =
		'data-[active=true]:nav-active-bg data-[active=true]:text-[--nav-active-text] ' +
		'data-[active=true]:ring-1 data-[active=true]:ring-[--nav-active-ring] ' +
		'data-[active=true]:border-[--nav-active-ring]';
</script>

{#if open}
	<div class="fixed inset-0 z-40 bg-black/40 md:hidden" onclick={close} aria-hidden="true"></div>
{/if}

<aside
	class={`fixed inset-y-0 left-0 z-50 w-72
          transform border-r border-[var(--ui-border)]
          bg-[var(--color-surface)] shadow-[var(--shadow-sidebar)] transition-transform
          md:static md:z-auto md:w-auto md:translate-x-0
          ${open ? 'translate-x-0' : '-translate-x-full md:translate-x-0'}
          ${collapsed ? 'md:w-[4.25rem]' : 'md:w-[18rem]'}
          ${rootClass}`}
	aria-label="Sidebar"
>
	<div class="flex h-dvh flex-col overflow-y-auto pb-4">
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

		<!-- Divider: MENU -->
		<div class={`px-4 pt-3 ${collapsed ? 'md:hidden' : ''}`}>
			<div class="text-xs tracking-wide uppercase opacity-60">Menu</div>
		</div>

		<!-- NAV group: Menu -->
		<nav class="mt-2 space-y-1 px-2">
			<!-- Dashboard -->
			<a
				href="/"
				data-active={isActive('/')}
				class={`${baseItem} ${activeMore}`}
				title="Dashboard"
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
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Dashboard</span>
			</a>

			<!-- Products -->
			<a
				href="/products"
				data-active={isActive('/products')}
				class={`${baseItem} ${activeMore}`}
				title="Products"
				onclick={close}
			>
				<svg
					viewBox="0 0 24 24"
					class="h-5 w-5 opacity-80"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
				>
					<rect x="3" y="3" width="7.5" height="7.5" rx="1.2" />
					<rect x="13.5" y="3" width="7.5" height="7.5" rx="1.2" />
					<rect x="3" y="13.5" width="7.5" height="7.5" rx="1.2" />
					<rect x="13.5" y="13.5" width="7.5" height="7.5" rx="1.2" />
				</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Products</span>
			</a>

			<!-- Data Table -->
			<a
				href="/tables/data"
				data-active={isActive('/tables/data')}
				class={`${baseItem} ${activeMore}`}
				title="Data Table"
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
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Data Table</span>
			</a>

			<!-- Settings -->
			<a
				href="/settings"
				data-active={isActive('/settings')}
				class={`${baseItem} ${activeMore}`}
				title="Settings"
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
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Settings</span>
			</a>
		</nav>

		<!-- Divider: OTHERS -->
		<div class={`px-4 pt-5 ${collapsed ? 'md:hidden' : ''}`}>
			<div class="text-xs tracking-wide uppercase opacity-60">Others</div>
		</div>

		<!-- NAV group: Components -->
		<nav class="mt-2 space-y-1 px-2">
			<!-- Components (parent) -->
			<a
				href="/components"
				data-active={isActive('/components')}
				class={`${baseItem} ${activeMore}`}
				title="Components"
				onclick={close}
			>
				<svg
					viewBox="0 0 24 24"
					class="h-5 w-5 opacity-80"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
				>
					<path d="M12 3l7 4v6l-7 4-7-4V7l7-4z" />
				</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Components</span>
			</a>

			<!-- Buttons -->
			<a
				href="/components/buttons"
				data-active={isActive('/components/buttons')}
				class={`${baseItem} ${activeMore} pl-9`}
				title="Buttons"
				onclick={close}
			>
				<svg
					viewBox="0 0 24 24"
					class="h-4 w-4 opacity-80"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
				>
					<rect x="3" y="8" width="18" height="8" rx="4" />
				</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Buttons</span>
			</a>

			<!-- Alerts -->
			<a
				href="/components/alerts"
				data-active={isActive('/components/alerts')}
				class={`${baseItem} ${activeMore} pl-9`}
				title="Alerts"
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
						d="M12 9v4m0 4h.01M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.7 3.86a2 2 0 0 0-3.42 0Z"
					/>
				</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Alerts</span>
			</a>

			<!-- Cards -->
			<a
				href="/components/cards"
				data-active={isActive('/components/cards')}
				class={`${baseItem} ${activeMore} pl-9`}
				title="Cards"
				onclick={close}
			>
				<svg
					viewBox="0 0 24 24"
					class="h-4 w-4 opacity-80"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
				>
					<rect x="3" y="5" width="18" height="14" rx="2" />
					<path d="M3 10h18" />
				</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Cards</span>
			</a>
		</nav>

		<!-- Divider: AUTH -->
		<div class={`px-4 pt-5 ${collapsed ? 'md:hidden' : ''}`}>
			<div class="text-xs tracking-wide uppercase opacity-60">Auth</div>
		</div>

		<!-- NAV group: Auth -->
		<nav class="mt-2 space-y-1 px-2">
			<a
				href="/auth/login"
				data-active={isActive('/auth/login')}
				class={`${baseItem} ${activeMore}`}
				title="Login"
				onclick={close}
			>
				<svg
					viewBox="0 0 24 24"
					class="h-5 w-5 opacity-80"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
				>
					<path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
					<path d="M10 17l5-5-5-5" />
					<path d="M15 12H3" />
				</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Login</span>
			</a>

			<a
				href="/auth/register"
				data-active={isActive('/auth/register')}
				class={`${baseItem} ${activeMore}`}
				title="Register"
				onclick={close}
			>
				<svg
					viewBox="0 0 24 24"
					class="h-5 w-5 opacity-80"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
				>
					<path d="M12 12a5 5 0 1 0-5-5 5 5 0 0 0 5 5Z" />
					<path d="M20 21a8 8 0 0 0-16 0" />
					<path d="M19 8v6m3-3h-6" />
				</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Register</span>
			</a>

			<a
				href="/auth/logout"
				data-active={isActive('/auth/logout')}
				class={`${baseItem} ${activeMore}`}
				title="Logout"
				onclick={close}
			>
				<svg
					viewBox="0 0 24 24"
					class="h-5 w-5 opacity-80"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
				>
					<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
					<path d="M14 7l5 5-5 5" />
					<path d="M19 12H7" />
				</svg>
				<span class={`truncate ${collapsed ? 'md:hidden' : ''}`}>Logout</span>
			</a>
		</nav>
	</div>
</aside>
