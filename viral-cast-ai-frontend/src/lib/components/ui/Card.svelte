<script lang="ts">
	import type { Snippet } from 'svelte';

	export type CardVariant = 'elevated' | 'soft' | 'outline' | 'ghost';
	export type CardImgPos = 'top' | 'left' | 'right' | 'none';

	// ⬇️ SATU kali $props(), sekalian ambil children (Snippet)
	let {
		title,
		subtitle,
		imgSrc,
		imgAlt = '',
		imgPosition = 'top' as CardImgPos,
		variant = 'elevated' as CardVariant,
		accent = 'var(--accent)',
		href,
		compact = false,
		header,
		actions,
		media,
		footer,
		children,
		badge,
		class: className = ''
	}: {
		title?: string;
		subtitle?: string;
		imgSrc?: string;
		imgAlt?: string;
		imgPosition?: CardImgPos;
		variant?: CardVariant;
		accent?: string;
		href?: string;
		compact?: boolean;
		header?: Snippet;
		actions?: Snippet;
		media?: Snippet;
		footer?: Snippet;
		children?: Snippet;
		badge?: Snippet;
		class?: string;
	} = $props();

	const basePad = $derived(compact ? 'p-4' : 'p-6');

	const layout = $derived.by(() =>
		imgPosition === 'left' || imgPosition === 'right'
			? 'split'
			: imgPosition === 'none'
				? 'content'
				: 'top'
	);

	const container = $derived.by(() =>
		[
			'group/card isolate overflow-hidden rounded-[var(--radius-card)]',
			'border border-[--card-border]',
			'bg-[--card-bg] text-[--color-text]',
			'shadow-[var(--card-shadow)] transition-shadow duration-200',
			'hover:shadow-[var(--card-shadow-hover)]',
			className
		].join(' ')
	);

	const styleVars = $derived.by(() => {
		const acc = accent ?? 'var(--accent)';
		switch (variant) {
			case 'soft':
				return [
					`--card-bg: color-mix(in oklab, ${acc} 7%, var(--color-surface));`,
					`--card-border: color-mix(in oklab, ${acc} 18%, transparent);`,
					`--card-shadow: var(--shadow-card);`,
					`--card-shadow-hover: var(--shadow-header);`
				].join('');
			case 'outline':
				return [
					`--card-bg: var(--color-surface);`,
					`--card-border: var(--ui-border);`,
					`--card-shadow: var(--shadow-card);`,
					`--card-shadow-hover: var(--shadow-header);`
				].join('');
			case 'ghost':
				return [
					`--card-bg: transparent;`,
					`--card-border: color-mix(in oklab, var(--ui-border) 60%, transparent);`,
					`--card-shadow: none;`,
					`--card-shadow-hover: var(--shadow-card);`
				].join('');
			default:
				return [
					`--card-bg: var(--color-surface);`,
					`--card-border: var(--ui-border);`,
					`--card-shadow: var(--shadow-card);`,
					`--card-shadow-hover: var(--shadow-header);`
				].join('');
		}
	});

	const Wrapper = $derived(href ? ('a' as const) : ('div' as const));
</script>

<svelte:element this={Wrapper} {href} class={container} style={styleVars}>
	{#if layout === 'split'}
		<div
			class={`flex flex-col md:flex-row ${imgPosition === 'right' ? 'md:flex-row-reverse' : ''}`}
		>
			<figure class="h-48 w-full bg-black/5 md:h-auto md:w-[40%] dark:bg-white/5">
				{#if media}
					{@render media?.()}
				{:else if imgSrc}
					<img src={imgSrc} alt={imgAlt} class="h-full w-full object-cover" />
				{/if}

				{#if badge}
					<div class="absolute top-3 left-3 z-10">
						{@render badge?.()}
					</div>
				{/if}
			</figure>

			<div class={`w-full md:flex-1 ${basePad}`}>
				{#if header}{@render header?.()}{/if}

				{#if title || subtitle || actions}
					<div class="mb-3 flex items-start justify-between gap-3">
						<div class="min-w-0">
							{#if title}<h3 class="truncate text-base font-semibold">{title}</h3>{/if}
							{#if subtitle}<p class="mt-0.5 text-sm opacity-70">{subtitle}</p>{/if}
						</div>
						{#if actions}<div class="shrink-0">{@render actions?.()}</div>{/if}
					</div>
				{/if}

				{@render children?.()}

				{#if footer}
					<div class="mt-4 border-t border-[--card-border] pt-4">
						{@render footer?.()}
					</div>
				{/if}
			</div>
		</div>
	{:else}
		{#if imgPosition === 'top' && (imgSrc || media)}
			<figure class="relative max-h-64 overflow-hidden">
				{#if media}
					{@render media?.()}
				{:else}
					<img src={imgSrc!} alt={imgAlt} class="h-56 w-full object-cover" />
				{/if}

				{#if badge}
					<div class="absolute top-3 left-3 z-10">
						{@render badge?.()}
					</div>
				{/if}
			</figure>
		{/if}

		<div class={basePad}>
			{#if header}{@render header?.()}{/if}

			{#if title || subtitle || actions}
				<div class="mb-3 flex items-start justify-between gap-3">
					<div class="min-w-0">
						{#if title}<h3 class="truncate text-base font-semibold">{title}</h3>{/if}
						{#if subtitle}<p class="mt-0.5 text-sm opacity-70">{subtitle}</p>{/if}
					</div>
					{#if actions}<div class="shrink-0">{@render actions?.()}</div>{/if}
				</div>
			{/if}

			{@render children?.()}

			{#if footer}
				<div class="mt-4 border-t border-[--card-border] pt-4">
					{@render footer?.()}
				</div>
			{/if}
		</div>
	{/if}
</svelte:element>
