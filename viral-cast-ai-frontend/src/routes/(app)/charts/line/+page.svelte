<script lang="ts">
	import LineChart from '$lib/components/charts/LineChart.svelte';

	const labels = [
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
	const data = [35, 50, 45, 60, 58, 75, 70, 80, 65, 72, 78, 90];
</script>

<section class="grid gap-6">
	<div
		class="rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-card)]"
	>
		<div class="mb-4">
			<h2 class="text-base font-semibold">Line Chart</h2>
			<p class="text-sm opacity-70">Trends over time</p>
		</div>
		<LineChart {labels} {data} />
	</div>
</section>
