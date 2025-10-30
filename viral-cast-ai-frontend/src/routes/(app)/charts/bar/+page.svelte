<script lang="ts">
	import BarChart from '$lib/components/charts/BarChart.svelte';

	const categories = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
	const series = [120, 90, 150, 110, 180, 140, 170];
</script>

<section class="grid gap-6">
	<div
		class="rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-card)]"
	>
		<div class="mb-4">
			<h2 class="text-base font-semibold">Bar Chart</h2>
			<p class="text-sm opacity-70">Category comparison</p>
		</div>
		<BarChart {categories} {series} />
	</div>
</section>
