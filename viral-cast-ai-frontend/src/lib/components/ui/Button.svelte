<!-- src/lib/components/ui/Button.svelte -->
<script lang="ts">
	import type { Snippet } from 'svelte';

	export type ButtonColor =
		| 'accent'
		| 'ocean' // → blue
		| 'navy' // → dark blue / biru tua
		| 'emerald'
		| 'rose'
		| 'grape' // → violet/purple
		| 'amber'
		| 'yellow'
		| 'deepYellow' // deep yellow / mustard
		| 'yellowDark' // NEW dark yellow
		| 'sky'
		| 'violet'
		| 'gray'
		| 'red'
		| 'orange' // Svelte orange (default)
		| 'green' // success
		| 'cyan'
		| 'lime'
		| 'pink' // NEW
		| 'pinkDeep'; // NEW

	export type ButtonSize = 'sm' | 'md' | 'lg';

	let {
		label,
		color = 'accent' as ButtonColor,
		size = 'md' as ButtonSize,
		disabled = false,
		href,
		type = 'button' as 'button' | 'submit' | 'reset',
		left,
		right,
		onClick
	}: {
		label: string;
		color?: ButtonColor;
		size?: ButtonSize;
		disabled?: boolean;
		href?: string;
		type?: 'button' | 'submit' | 'reset';
		left?: Snippet;
		right?: Snippet;
		onClick?: () => void;
	} = $props();

	// SATU peta warna → CSS var (sinkron dgn app.css)
	const COLOR_MAP: Record<ButtonColor, string> = {
		accent: 'var(--accent)',
		ocean: 'var(--color-blue-600)',
		navy: 'var(--color-blue-dark-600)',
		emerald: 'var(--color-emerald-600)',
		rose: 'var(--color-rose-600)',
		grape: 'var(--color-violet-600)',
		amber: 'var(--color-amber-600)',
		yellow: 'var(--color-yellow-600)',
		deepYellow: 'var(--color-yellow-deep-600)',
		yellowDark: 'var(--color-yellow-dark-600)',
		sky: 'var(--color-sky-600)',
		violet: 'var(--color-violet-600)',
		gray: 'var(--color-gray-600)',
		red: 'var(--color-red-600)',

		orange: 'var(--color-orange-deep-600)',

		green: 'var(--color-green-600)',
		cyan: 'var(--color-cyan-600)',
		lime: 'var(--color-lime-600)',
		pink: 'var(--color-pink-600)', // NEW
		pinkDeep: 'var(--color-pink-deep-600)' // NEW (pink pekat)
	};

	// css var untuk background (string, bukan function)
	const btnBg = $derived.by(() => COLOR_MAP[color]);

	const sizeCls = $derived.by(() => {
		switch (size) {
			case 'sm':
				return 'text-xs px-3 py-1.5';
			case 'lg':
				return 'text-base px-5 py-3';
			default:
				return 'text-sm px-4 py-2';
		}
	});

	const baseCls =
		'inline-flex items-center gap-2 rounded-[var(--radius-pill)] ' +
		'text-white shadow-card ring-1 ring-white/10 ' +
		'hover:brightness-105 active:brightness-[.95] ' +
		'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/40 ' +
		'disabled:opacity-50 disabled:cursor-not-allowed select-none';

	function handleClick() {
		if (disabled) return;
		onClick?.();
	}
</script>

{#if href}
	<a
		{href}
		aria-disabled={disabled}
		class={`${baseCls} ${sizeCls} ${color === 'accent' ? 'bg-[--accent]' : ''}`}
		style={`background: var(--btn-bg);`}
		style:--btn-bg={btnBg}
		onclick={handleClick}
	>
		{#if left}{@render left?.()}{/if}
		<span class="whitespace-nowrap">{label}</span>
		{#if right}{@render right?.()}{/if}
	</a>
{:else}
	<button
		{type}
		{disabled}
		class={`${baseCls} ${sizeCls} ${color === 'accent' ? 'bg-[--accent]' : ''}`}
		style={`background: var(--btn-bg);`}
		style:--btn-bg={btnBg}
		onclick={handleClick}
	>
		{#if left}{@render left?.()}{/if}
		<span class="whitespace-nowrap">{label}</span>
		{#if right}{@render right?.()}{/if}
	</button>
{/if}

<style>
	/* Shadow util – ikut tokens app.css bila ada */
	.shadow-card {
		box-shadow: var(--shadow-card, 0 1px 2px oklch(0 0 0 / 0.05), 0 8px 24px oklch(0 0 0 / 0.08));
	}
</style>
