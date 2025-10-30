<script lang="ts">
	import { onMount } from 'svelte';

	export let categories: string[] = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
	export let series: number[] = [120, 90, 150, 110, 180, 140, 170];
	export let height = 320;

	let host: HTMLElement | null = null;
	let width = 640;

	const margin = { top: 16, right: 16, bottom: 42, left: 44 };

	function updateSize() {
		const w = (host as any)?.getBoundingClientRect?.().width ?? (host as any)?.clientWidth ?? 640;
		width = Math.max(320, Math.round(w));
	}
	onMount(() => {
		updateSize();
		const g: any = globalThis as any;
		const RO = g?.ResizeObserver;
		if (RO) {
			const ro = new RO(updateSize);
			(host as any) && ro.observe(host);
			return () => ro.disconnect();
		} else {
			const onR = () => updateSize();
			g?.addEventListener?.('resize', onR);
			return () => g?.removeEventListener?.('resize', onR);
		}
	});

	$: innerW = Math.max(0, width - margin.left - margin.right);
	$: innerH = Math.max(0, height - margin.top - margin.bottom);
	$: maxY = Math.max(10, Math.max(...series) * 1.15);

	function sx(i: number) {
		const gap = 12;
		const bw = (innerW - gap * (categories.length - 1)) / categories.length;
		return margin.left + i * (bw + gap);
	}
	function barWidth() {
		const gap = 12;
		return (innerW - gap * (categories.length - 1)) / categories.length;
	}
	function sy(v: number) {
		const t = v / maxY;
		return margin.top + innerH - t * innerH;
	}

	let tipIdx: number | null = null;
	let tipX = 0;
	let tipY = 0;

	function showTip(i: number) {
		tipIdx = i;
		tipX = sx(i) + barWidth() / 2;
		tipY = sy(series[i]) - 8;
	}
	function hideTip() {
		tipIdx = null;
	}
</script>

<div bind:this={host} class="relative w-full">
	<svg {height} {width} viewBox={`0 0 ${width} ${height}`} class="w-full">
		<!-- grid lines -->
		{#each Array.from({ length: 5 }, (_, i) => i) as i}
			<line
				x1={margin.left}
				x2={width - margin.right}
				y1={margin.top + (i * innerH) / 4}
				y2={margin.top + (i * innerH) / 4}
				stroke="currentColor"
				class="opacity-10"
			/>
		{/each}

		<!-- axes -->
		<line
			x1={margin.left}
			y1={margin.top + innerH}
			x2={width - margin.right}
			y2={margin.top + innerH}
			stroke="currentColor"
			class="opacity-20"
		/>
		<line
			x1={margin.left}
			y1={margin.top}
			x2={margin.left}
			y2={margin.top + innerH}
			stroke="currentColor"
			class="opacity-20"
		/>

		<!-- bars -->
		{#each series as v, i}
			<rect
				x={sx(i)}
				y={sy(v)}
				width={barWidth()}
				height={margin.top + innerH - sy(v)}
				class="fill-[color-mix(in_oklch,var(--accent),transparent_30%)] stroke-[--accent]"
				stroke-width="1.5"
				role="presentation"
				aria-hidden="true"
				on:mousemove={() => showTip(i)}
				on:mouseleave={hideTip}
			/>
		{/each}

		<!-- labels -->
		{#each categories as c, i}
			<text
				x={sx(i) + barWidth() / 2}
				y={margin.top + innerH + 24}
				text-anchor="middle"
				class="fill-current text-xs opacity-70 select-none">{c}</text
			>
		{/each}
	</svg>

	{#if tipIdx !== null}
		<div
			class="pointer-events-none absolute z-10 -translate-x-1/2 -translate-y-3 rounded-md border border-[var(--ui-border)] bg-[var(--color-surface)] px-2 py-1 text-xs shadow-[var(--shadow-card)]"
			style={`left:${tipX}px; top:${tipY}px`}
		>
			<div class="font-medium">{categories[tipIdx]}</div>
			<div class="opacity-70">{series[tipIdx]}</div>
		</div>
	{/if}
</div>
