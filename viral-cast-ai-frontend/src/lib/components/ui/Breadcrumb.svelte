<script lang="ts">
	import type { Snippet } from 'svelte';

	type Crumb = { label: string; href?: string };

	let {
		items = [] as Crumb[],
		separator = 'chevron' as 'chevron' | 'slash' | 'dot',
		size = 'sm' as 'sm' | 'md',
		full = false,
		prefix, // <- boleh ada, boleh tidak
		class: className = ''
	}: {
		items?: Crumb[];
		separator?: 'chevron' | 'slash' | 'dot';
		size?: 'sm' | 'md';
		full?: boolean;
		prefix?: Snippet; // <- jadikan OPTIONAL
		class?: string;
	} = $props();

	const sizeCls = $derived(size === 'md' ? 'text-sm gap-2' : 'text-xs gap-1.5');

	const wrapCls = $derived(
		[
			'text-[--color-text]',
			full
				? 'rounded-[var(--radius-card)] border border-[--ui-border] bg-[--color-surface] px-3 py-2'
				: '',
			className
		].join(' ')
	);

	const linkCls =
		'inline-flex items-center hover:text-[--accent] focus-visible:outline-none focus-visible:ring-2 ' +
		'focus-visible:ring-[--nav-active-ring] rounded px-0.5';
	const currentCls = 'opacity-70';
</script>

<nav aria-label="Breadcrumb" class={wrapCls}>
	<ol class={`flex flex-wrap items-center ${sizeCls}`}>
		{#if prefix}
			<li class="shrink-0">
				{@render prefix?.()}
			</li>
			{#if items.length}
				<li aria-hidden="true" class="opacity-50">
					{#if separator === 'chevron'}
						<svg
							viewBox="0 0 20 20"
							width="16"
							height="16"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M7 5l6 5-6 5" />
						</svg>
					{:else if separator === 'slash'}
						/
					{:else}
						•
					{/if}
				</li>
			{/if}
		{/if}

		{#each items as c, i (c.label)}
			<li class="shrink-0">
				{#if c.href && i < items.length - 1}
					<a href={c.href} class={linkCls}>{c.label}</a>
				{:else}
					<span aria-current="page" class={currentCls}>{c.label}</span>
				{/if}
			</li>

			{#if i < items.length - 1}
				<li aria-hidden="true" class="opacity-50">
					{#if separator === 'chevron'}
						<svg
							viewBox="0 0 20 20"
							width="16"
							height="16"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M7 5l6 5-6 5" />
						</svg>
					{:else if separator === 'slash'}
						/
					{:else}
						•
					{/if}
				</li>
			{/if}
		{/each}
	</ol>
</nav>
