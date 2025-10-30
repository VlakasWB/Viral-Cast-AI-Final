<script lang="ts">
	import { onMount } from 'svelte';

	export let labels: string[] = [
		'Jan',
		'Feb',
		'Mar',
		'Apr',
		'May',
		'Jun',
		'Jul',
		'Aug',
		'Sep',
		'Oct',
		'Nov',
		'Dec'
	];
	export let data: number[] = [35, 50, 45, 60, 58, 75, 70, 80, 65, 72, 78, 90];
	export let height = 320;

	// pakai HTMLElement | null + guard any agar aman walau lib DOM tdk tersedia
	let host: HTMLElement | null = null;
	let width = 640;

	const margin = { top: 16, right: 16, bottom: 42, left: 44 };

	function updateSize() {
		// guard ?. dan cast any → aman meski TS tidak tahu DOM
		const w = (host as any)?.getBoundingClientRect?.().width ?? (host as any)?.clientWidth ?? 640;
		width = Math.max(320, Math.round(w));
	}

	onMount(() => {
		updateSize();
		const g: any = globalThis as any;
		const RO = g?.ResizeObserver;
		if (RO) {
			const ro = new RO(() => updateSize());
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
	$: maxY = Math.max(10, Math.max(...data) * 1.15);
	$: minY = 0;

	function sx(i: number) {
		if (labels.length <= 1) return margin.left;
		return margin.left + (i * innerW) / (labels.length - 1);
	}
	function sy(v: number) {
		const t = (v - minY) / (maxY - minY);
		return margin.top + innerH - t * innerH;
	}

	$: path = data.map((v, i) => `${i === 0 ? 'M' : 'L'} ${sx(i)} ${sy(v)}`).join(' ');

	$: area = [
		`M ${sx(0)} ${sy(data[0])}`,
		...data.slice(1).map((v, i) => `L ${sx(i + 1)} ${sy(v)}`),
		`L ${sx(data.length - 1)} ${margin.top + innerH}`,
		`L ${sx(0)} ${margin.top + innerH}`,
		'Z'
	].join(' ');

	// tooltip
	let tipIdx: number | null = null;
	let tipX = 0;
	let tipY = 0;

	// HINDARI clientX/DOM typing ketat → pakai hit-test terhadap sumbu X terdekat
	function moveTip(e: any) {
		// fallback koordinat relatif: ambil posisi mouse dari offsetX jika ada
		const svg = e?.currentTarget as any;
		const rect = svg?.getBoundingClientRect?.();
		const x = typeof e?.clientX === 'number' && rect ? e.clientX - rect.left : (e?.offsetX ?? 0);

		let nearest = 0;
		let nearestDx = Infinity;
		for (let i = 0; i < labels.length; i++) {
			const dx = Math.abs(sx(i) - x);
			if (dx < nearestDx) {
				nearestDx = dx;
				nearest = i;
			}
		}
		tipIdx = nearest;
		tipX = sx(nearest);
		tipY = sy(data[nearest]);
	}
	function leaveTip() {
		tipIdx = null;
	}
</script>

<div bind:this={host} class="relative w-full">
	<svg {height} {width} viewBox={`0 0 ${width} ${height}`} class="w-full">
		<!-- grid Y -->
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

		<!-- area -->
		<path d={area} class="fill-[color-mix(in_oklch,var(--accent),transparent_85%)]" />

		<!-- line -->
		<path d={path} class="stroke-[--accent]" stroke-width="2.5" fill="none" />

		<!-- circles -->
		{#each data as v, i}
			<circle cx={sx(i)} cy={sy(v)} r="3" class="fill-[--accent]" />
		{/each}

		<!-- labels X -->
		{#each labels as lbl, i}
			<text
				x={sx(i)}
				y={margin.top + innerH + 24}
				text-anchor="middle"
				class="fill-current text-xs opacity-70 select-none">{lbl}</text
			>
		{/each}

		<!-- pointer layer (a11y role agar tidak kena warning) -->
		<rect
			x="0"
			y="0"
			{width}
			{height}
			class="cursor-crosshair fill-transparent"
			role="presentation"
			aria-hidden="true"
			on:mousemove={moveTip}
			on:mouseleave={leaveTip}
		/>
	</svg>

	{#if tipIdx !== null}
		<div
			class="pointer-events-none absolute z-10 -translate-x-1/2 -translate-y-3 rounded-md border border-[var(--ui-border)] bg-[var(--color-surface)] px-2 py-1 text-xs shadow-[var(--shadow-card)]"
			style={`left:${tipX}px; top:${tipY}px`}
		>
			<div class="font-medium">{labels[tipIdx]}</div>
			<div class="opacity-70">{data[tipIdx]}</div>
		</div>
	{/if}
</div>
