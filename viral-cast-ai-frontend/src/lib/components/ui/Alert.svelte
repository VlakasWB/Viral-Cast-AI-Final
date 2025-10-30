<!-- src/lib/components/ui/Alert.svelte -->
<script lang="ts">
	import type { Snippet } from 'svelte';

	// Type cukup lokal (tidak di-export) agar tidak bentrok props di Svelte 5
	type AlertKind =
		| 'primary' // blue
		| 'secondary' // gray
		| 'success' // green
		| 'warning' // amber
		| 'danger' // red
		| 'info' // sky
		| 'accent'; // follow var(--accent)

	type AlertVariant = 'soft' | 'solid' | 'outline';

	let {
		kind = 'primary' as AlertKind,
		variant = 'soft' as AlertVariant,
		title = '',
		description = '',
		dismissible = false,
		onClose,
		class: className = '',
		icon // optional Snippet override untuk ikon
	}: {
		kind?: AlertKind;
		variant?: AlertVariant;
		title?: string;
		description?: string;
		dismissible?: boolean;
		onClose?: () => void;
		class?: string;
		icon?: Snippet;
	} = $props();

	let open = $state(true);

	// Peta warna → token CSS yang sudah ada di app.css
	const COLOR_VAR: Record<AlertKind, string> = {
		primary: 'var(--color-blue-600)',
		secondary: 'var(--color-gray-600)',
		success: 'var(--color-green-600)',
		warning: 'var(--color-amber-600)',
		danger: 'var(--color-red-600)',
		info: 'var(--color-sky-600)',
		accent: 'var(--accent)'
	};

	const base = $derived(COLOR_VAR[kind]);

	// Variant → bg / fg / border
	const bg = $derived(
		variant === 'solid'
			? base
			: variant === 'soft'
				? `oklch(from ${base} l c h / 0.12)`
				: 'transparent'
	);
	const fg = $derived(variant === 'solid' ? 'white' : base);
	const bd = $derived(
		variant === 'outline'
			? base
			: `oklch(from ${base} l c h / ${variant === 'soft' ? '0.28' : '0.35'})`
	);

	function close() {
		open = false;
		onClose?.();
	}
</script>

{#if open}
	<div
		class={`shadow-card relative flex gap-3 rounded-xl border border-[var(--alert-bd)] bg-[var(--alert-bg)]
          p-3 text-[var(--alert-fg)] md:p-4 ${className}`}
		style:--alert-bg={bg}
		style:--alert-fg={fg}
		style:--alert-bd={bd}
	>
		<!-- Icon -->
		<div class="mt-0.5 shrink-0 opacity-90">
			{#if icon}
				{@render icon()}
			{:else if kind === 'success'}
				<svg
					viewBox="0 0 24 24"
					width="18"
					height="18"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path d="M20 6L9 17l-5-5" />
				</svg>
			{:else if kind === 'warning'}
				<svg
					viewBox="0 0 24 24"
					width="18"
					height="18"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path d="M12 9v4M12 17h.01" />
					<path
						d="M10.3 2.1 1.8 17a2 2 0 0 0 1.7 3h16.9a2 2 0 0 0 1.7-3L13.7 2.1a2 2 0 0 0-3.4 0Z"
					/>
				</svg>
			{:else if kind === 'danger'}
				<svg
					viewBox="0 0 24 24"
					width="18"
					height="18"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path d="M18 6 6 18M6 6l12 12" />
				</svg>
			{:else if kind === 'info'}
				<svg
					viewBox="0 0 24 24"
					width="18"
					height="18"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<circle cx="12" cy="12" r="10" />
					<path d="M12 8h.01M11 12h2v4h-2z" />
				</svg>
			{:else if kind === 'secondary'}
				<svg
					viewBox="0 0 24 24"
					width="18"
					height="18"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path d="M4 4h16v16H4z" />
				</svg>
			{:else}
				<!-- primary / accent -->
				<svg
					viewBox="0 0 24 24"
					width="18"
					height="18"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path d="M12 2v20M2 12h20" />
				</svg>
			{/if}
		</div>

		<!-- Text -->
		<div class="min-w-0">
			{#if title}<div class="leading-6 font-semibold">{title}</div>{/if}
			{#if description}<div class="mt-0.5 text-sm opacity-90">{description}</div>{/if}
		</div>

		<!-- Close -->
		{#if dismissible}
			<button
				type="button"
				class="absolute top-2 right-2 rounded-md p-1 opacity-70 transition
               hover:opacity-100 focus-visible:ring-2
               focus-visible:ring-[--alert-bd] focus-visible:outline-none"
				aria-label="Close"
				onclick={close}
			>
				<svg
					viewBox="0 0 24 24"
					width="18"
					height="18"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path d="M18 6 6 18M6 6l12 12" />
				</svg>
			</button>
		{/if}
	</div>
{/if}
