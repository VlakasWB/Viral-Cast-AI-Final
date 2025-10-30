<script lang="ts">
	import PieChart from '$lib/components/charts/PieChart.svelte';

	const labels = ['Desktop', 'Tablet', 'Mobile'];
	const values = [50, 20, 30];
</script>

<section class="grid gap-6">
	<div
		class="rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-card)]"
	>
		<div class="mb-4">
			<h2 class="text-base font-semibold">Pie Chart</h2>
			<p class="text-sm opacity-70">Traffic sources</p>
		</div>
		<PieChart {labels} {values} />
	</div>
</section>
