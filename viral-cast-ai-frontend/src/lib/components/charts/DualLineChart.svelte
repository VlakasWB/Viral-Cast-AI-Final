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
	export let realData: number[] = [35, 50, 45, 60, 58, 75, 70, 80, 65, 72, 78, 90];
	export let predictionData: number[] = [40, 55, 50, 65, 63, 80, 75, 85, 70, 77, 83, 95];
	export let height = 320;
	export let realColor = '#E98621'; // warna oranye sesuai permintaan
	export let predictionColor = '#3b82f6'; // blue-500

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
	$: allData = [...realData, ...predictionData];
	$: maxY = Math.max(10, Math.max(...allData) * 1.15);
	$: minY = 0;

	function sx(i: number) {
		if (labels.length <= 1) return margin.left;
		return margin.left + (i * innerW) / (labels.length - 1);
	}

	function sy(v: number) {
		const t = (v - minY) / (maxY - minY);
		return margin.top + innerH - t * innerH;
	}

	// Generate path for real data (solid line)
	$: realPath = realData.map((v, i) => `${i === 0 ? 'M' : 'L'} ${sx(i)} ${sy(v)}`).join(' ');

	// Generate path for prediction data (dashed line)
	$: predictionPath = predictionData
		.map((v, i) => `${i === 0 ? 'M' : 'L'} ${sx(i)} ${sy(v)}`)
		.join(' ');

	// Tooltip state
	let tipIdx: number | null = null;
	let tipX = 0;
	let tipY = 0;
	let tipType: 'real' | 'prediction' | null = null;

	function showTip(i: number, type: 'real' | 'prediction', event?: MouseEvent | KeyboardEvent) {
		tipIdx = i;
		tipType = type;
		tipX = sx(i);
		tipY = sy(type === 'real' ? realData[i] : predictionData[i]) - 8;
	}

	function hideTip() {
		tipIdx = null;
		tipType = null;
	}

	// Format currency to Indonesian Rupiah
	function formatRupiah(amount: number): string {
		return new Intl.NumberFormat('id-ID', {
			style: 'currency',
			currency: 'IDR',
			minimumFractionDigits: 0,
			maximumFractionDigits: 0
		}).format(amount * 1000000); // Convert to millions
	}
</script>

<div bind:this={host} class="relative w-full">
	<svg {height} {width} viewBox={`0 0 ${width} ${height}`} class="w-full">
		<!-- Grid lines -->
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

		<!-- Axes -->
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

		<!-- Real data line (solid) -->
		<path
			d={realPath}
			fill="none"
			stroke={realColor}
			stroke-width="3"
			stroke-linecap="round"
			stroke-linejoin="round"
		/>

		<!-- Prediction data line (dashed) -->
		<path
			d={predictionPath}
			fill="none"
			stroke={predictionColor}
			stroke-width="3"
			stroke-linecap="round"
			stroke-linejoin="round"
			stroke-dasharray="8,4"
		/>

		<!-- Data points for real data -->
		{#each realData as v, i}
			<circle
				cx={sx(i)}
				cy={sy(v)}
				r="4"
				fill={realColor}
				stroke="white"
				stroke-width="2"
				class="cursor-pointer"
				role="button"
				tabindex="0"
				aria-label="Actual data {labels[i]}: {formatRupiah(v)}"
				on:mousemove={(e) => showTip(i, 'real', e)}
				on:mouseleave={hideTip}
				on:keydown={(e) => e.key === 'Enter' && showTip(i, 'real', e)}
			/>
		{/each}

		<!-- Data points for prediction data -->
		{#each predictionData as v, i}
			<circle
				cx={sx(i)}
				cy={sy(v)}
				r="4"
				fill={predictionColor}
				stroke="white"
				stroke-width="2"
				class="cursor-pointer"
				role="button"
				tabindex="0"
				aria-label="Prediction data {labels[i]}: {formatRupiah(v)}"
				on:mousemove={(e) => showTip(i, 'prediction', e)}
				on:mouseleave={hideTip}
				on:keydown={(e) => e.key === 'Enter' && showTip(i, 'prediction', e)}
			/>
		{/each}

		<!-- X-axis labels (months) -->
		{#each labels as label, i}
			<text
				x={sx(i)}
				y={margin.top + innerH + 24}
				text-anchor="middle"
				class="fill-current text-xs opacity-70 select-none"
			>
				{label}
			</text>
		{/each}

		<!-- Y-axis labels (revenue) -->
		{#each Array.from({ length: 5 }, (_, i) => i) as i}
			<text
				x={margin.left - 8}
				y={margin.top + (i * innerH) / 4 + 4}
				text-anchor="end"
				class="fill-current text-xs opacity-70 select-none"
			>
				{Math.round((maxY * (4 - i)) / 4)}M
			</text>
		{/each}
	</svg>

	<!-- Tooltip -->
	{#if tipIdx !== null && tipType !== null}
		<div
			class="pointer-events-none absolute z-10 -translate-x-1/2 -translate-y-3 rounded-md border border-[var(--ui-border)] bg-[var(--color-surface)] px-3 py-2 text-xs shadow-[var(--shadow-card)]"
			style={`left:${tipX}px; top:${tipY}px`}
		>
			<div class="font-medium">{labels[tipIdx]}</div>
			<div class="opacity-70" style="color: {tipType === 'real' ? realColor : predictionColor}">
				{tipType === 'real' ? 'Actual' : 'Prediction'}: {formatRupiah(
					tipType === 'real' ? realData[tipIdx] : predictionData[tipIdx]
				)}
			</div>
		</div>
	{/if}

	<!-- Legend -->
	<div class="absolute top-4 right-4 flex gap-4 text-xs">
		<div class="flex items-center gap-2">
			<div class="h-0.5 w-4" style="background-color: {realColor}"></div>
			<span>Actual Data</span>
		</div>
		<div class="flex items-center gap-2">
			<div class="h-0.5 w-4 border-t-2 border-dashed" style="border-color: {predictionColor}"></div>
			<span>Prediction</span>
		</div>
	</div>
</div>
