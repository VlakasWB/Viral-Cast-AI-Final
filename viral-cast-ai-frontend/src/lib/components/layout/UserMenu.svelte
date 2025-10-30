<script lang="ts">
	import { browser } from '$app/environment';

	// Props
	let { name = 'Your Name', email = 'you@example.com', avatarUrl = '' } = $props();

	// state
	let open = $state(false);
	let root: any = null; // pakai any supaya tidak tergantung DOM lib

	function toggle() {
		open = !open;
	}
	function close() {
		open = false;
	}

	// klik di luar + Esc (aman-SSR + tanpa tipe DOM)
	$effect(() => {
		if (!browser) return;
		const doc: any = (globalThis as any).document;
		if (!doc) return;

		const onDocClick = (e: any) => {
			if (!open || !root) return;
			const t: any = e?.target;
			// cast any untuk menghindari error .contains
			if (root && !(root as any).contains(t)) close();
		};
		const onKey = (e: any) => {
			if (e?.key === 'Escape') close();
		};

		doc.addEventListener('click', onDocClick);
		doc.addEventListener('keydown', onKey);
		return () => {
			doc.removeEventListener('click', onDocClick);
			doc.removeEventListener('keydown', onKey);
		};
	});
</script>

<div class="relative" bind:this={root}>
	<!-- Trigger -->
	<button
		type="button"
		class="group shadow-header inline-flex items-center gap-2 rounded-full border border-[var(--ui-border)] bg-[var(--color-surface)] px-2.5 py-1.5 transition hover:opacity-100 focus-visible:ring-2 focus-visible:ring-[--nav-active-ring] focus-visible:outline-none"
		aria-haspopup="menu"
		aria-expanded={open}
		onclick={toggle}
	>
		<!-- Avatar -->
		{#if avatarUrl}
			<img
				src={avatarUrl}
				alt="User avatar"
				class="h-8 w-8 rounded-full object-cover"
				width="32"
				height="32"
			/>
		{:else}
			<!-- fallback avatar -->
			<div
				class="grid h-8 w-8 place-items-center rounded-full bg-[color-mix(in_oklch,var(--accent),transparent_82%)] text-[--accent]"
			>
				<svg width="16" height="16" viewBox="0 0 24 24" aria-hidden="true">
					<path
						fill="currentColor"
						d="M12 12a5 5 0 100-10 5 5 0 000 10zm0 2c-4.2 0-8 2.2-8 5v1h16v-1c0-2.8-3.8-5-8-5z"
					/>
				</svg>
			</div>
		{/if}

		<!-- Name -->
		<span class="hidden text-sm sm:block">{name}</span>

		<!-- chevron (gunakan data-open, bukan style:) -->
		<svg
			class="h-4 w-4 transition group-data-[open=true]:rotate-180"
			data-open={open ? 'true' : 'false'}
			viewBox="0 0 20 20"
			fill="currentColor"
			aria-hidden="true"
		>
			<path
				fill-rule="evenodd"
				d="M5.23 7.21a.75.75 0 011.06.02L10 10.94l3.71-3.71a.75.75 0 111.06 1.06l-4.24 4.24a.75.75 0 01-1.06 0L5.21 8.29a.75.75 0 01.02-1.08z"
				clip-rule="evenodd"
			/>
		</svg>
	</button>

	<!-- Dropdown -->
	{#if open}
		<div
			role="menu"
			aria-label="User menu"
			class="shadow-card absolute right-0 z-50 mt-2 w-64 origin-top-right rounded-2xl border border-[var(--ui-border)] bg-[var(--color-surface)] p-3 text-sm"
			style="--ring: var(--nav-active-ring);"
		>
			<!-- header -->
			<div class="mb-3 rounded-xl bg-[color-mix(in_oklch,var(--color-surface),black_4%)] p-3">
				<div class="font-medium">{name}</div>
				<div class="truncate text-xs opacity-70">{email}</div>
			</div>

			<ul class="space-y-1" role="none">
				<li role="none">
					<a
						role="menuitem"
						href="/settings"
						class="flex items-center gap-3 rounded-xl px-3 py-2 hover:bg-[color-mix(in_oklch,var(--color-surface),black_4%)] focus-visible:ring-2 focus-visible:ring-[--ring] focus-visible:outline-none"
						onclick={close}
					>
						<svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
							<path
								fill="currentColor"
								d="M12 8a4 4 0 100 8 4 4 0 000-8zm8.94 4a7.96 7.96 0 00-.34-2l2.05-1.6-2-3.46-2.5.5a8.06 8.06 0 00-1.7-1L15 1h-6l-.45 2.44c-.6.24-1.17.55-1.7.92l-2.5-.53-2 3.46 2.05 1.6c-.15.64-.25 1.32-.25 2.02 0 .7.1 1.38.27 2.02L.85 15.6l2 3.46 2.5-.53c.53.36 1.1.67 1.7.9L9 22h6l.45-2.44c.6-.23 1.17-.54 1.7-.9l2.5.53 2-3.46-2.05-1.6c.18-.64.29-1.32.29-2.03z"
							/>
						</svg>
						Settings
					</a>
				</li>

				<li role="none">
					<a
						role="menuitem"
						href="/support"
						class="flex items-center gap-3 rounded-xl px-3 py-2 hover:bg-[color-mix(in_oklch,var(--color-surface),black_4%)] focus-visible:ring-2 focus-visible:ring-[--ring] focus-visible:outline-none"
						onclick={close}
					>
						<svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
							<path
								fill="currentColor"
								d="M12 2a10 10 0 100 20 10 10 0 000-20zm1 15h-2v-2h2v2zm2.07-7.75l-.9.92A3.49 3.49 0 0013 12h-2v-.5c0-.8.32-1.56.88-2.12l1.24-1.26a1.5 1.5 0 10-2.12-2.12l-.62.62-1.42-1.42.62-.62a3.5 3.5 0 014.95 4.95z"
							/>
						</svg>
						Support
					</a>
				</li>

				<li class="my-2 border-t border-[var(--ui-border)]" role="separator"></li>

				<li role="none">
					<a
						role="menuitem"
						href="/logout"
						class="flex items-center gap-3 rounded-xl px-3 py-2 text-red-500 hover:bg-[color-mix(in_oklch,var(--color-surface),black_4%)] focus-visible:ring-2 focus-visible:ring-[--ring] focus-visible:outline-none"
						onclick={close}
					>
						<svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
							<path
								fill="currentColor"
								d="M16 13v-2H7V8l-5 4 5 4v-3h9zM20 3h-8v2h8v14h-8v2h8a2 2 0 002-2V5a2 2 0 00-2-2z"
							/>
						</svg>
						Sign out
					</a>
				</li>
			</ul>
		</div>
	{/if}
</div>
