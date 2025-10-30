<script lang="ts">
	export let labels: string[] = ['Desktop', 'Tablet', 'Mobile'];
	export let values: number[] = [50, 20, 30];
    // height is for external reference only; use const to avoid unused export warning
    export const height = 320;

	// warna mengikuti aksen, sisanya derivatif
	const fills = [
		'oklch(65% 0.15 var(--accent-h))',
		'oklch(from var(--accent) 65% 0.08 h)',
		'oklch(from var(--accent) 62% 0.06 h)'
	];

	const size = 320;
	const cx = size / 2;
	const cy = size / 2;
	const r = 110;

	$: total = values.reduce((a, b) => a + b, 0) || 1;

	function arc(start: number, end: number) {
		const large = end - start > Math.PI ? 1 : 0;
		const x1 = cx + r * Math.cos(start);
		const y1 = cy + r * Math.sin(start);
		const x2 = cx + r * Math.cos(end);
		const y2 = cy + r * Math.sin(end);
		return `M ${cx} ${cy} L ${x1} ${y1} A ${r} ${r} 0 ${large} 1 ${x2} ${y2} Z`;
	}

	$: arcs = (() => {
		let a = -Math.PI / 2;
		return values.map((v) => {
			const delta = (v / total) * Math.PI * 2;
			const path = arc(a, a + delta);
			const mid = a + delta / 2;
			a += delta;
			return { path, mid };
		});
	})();

	// tooltip
	let tipIdx: number | null = null;
	let tipX = 0;
	let tipY = 0;

	function showTip(i: number, e: MouseEvent) {
		tipIdx = i;
		// posisi tooltip di tengah slice
		tipX = cx + Math.cos(arcs[i].mid) * (r * 0.65);
		tipY = cy + Math.sin(arcs[i].mid) * (r * 0.65);
	}
	function hideTip() {
		tipIdx = null;
	}
</script>

<div class="grid gap-6 md:grid-cols-2">
	<div class="relative grid place-items-center">
		<svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} class="max-w-full">
			{#each arcs as a, i}
                <path role="img" aria-label={`Slice ${labels[i]}: ${values[i]}`}
                    d={a.path}
                    class="stroke-[var(--ui-border)]"
                    stroke-width="1.5"
                    fill={fills[i % fills.length]}
                    on:mousemove={(e) => showTip(i, e)}
                    on:mouseleave={hideTip}
                />
			{/each}
		</svg>

		{#if tipIdx !== null}
			<div
				class="pointer-events-none absolute z-10 -translate-x-1/2 -translate-y-1/2 rounded-md border border-[var(--ui-border)] bg-[var(--color-surface)] px-2 py-1 text-xs shadow-[var(--shadow-card)]"
				style={`left:${tipX}px; top:${tipY}px`}
			>
				<div class="font-medium">{labels[tipIdx]}</div>
				<div class="opacity-70">
					{values[tipIdx]} ({Math.round((values[tipIdx] / total) * 100)}%)
				</div>
			</div>
		{/if}
	</div>

	<ul class="space-y-3 self-center">
		{#each labels as lbl, i}
			<li class="flex items-center justify-between gap-4">
				<div class="flex items-center gap-3">
					<span
						class="inline-block h-3 w-3 rounded-sm"
						style={`background:${fills[i % fills.length]}`}
					></span>
					<span class="text-sm">{lbl}</span>
				</div>
				<span class="text-sm opacity-80">{values[i]}</span>
			</li>
		{/each}
	</ul>
</div>
