<script lang="ts">
	import Card from '$lib/components/Card.svelte';
	import DualLineChart from '$lib/components/charts/DualLineChart.svelte';
	import WeatherWidget from '$lib/components/WeatherWidget.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	// Format currency to Indonesian Rupiah
	function formatRupiah(amount: number): string {
		return new Intl.NumberFormat('id-ID', {
			style: 'currency',
			currency: 'IDR',
			minimumFractionDigits: 0,
			maximumFractionDigits: 0
		}).format(amount);
	}

	// Format number with thousand separators
	function formatNumber(num: number): string {
		return new Intl.NumberFormat('id-ID').format(num);
	}

	// Calculate percentage change (mock data for now)
	function getPercentageChange(): string {
		const changes = ['+12%', '+8%', '-3%', '+15%', '+5%'];
		return changes[Math.floor(Math.random() * changes.length)];
	}

	// Dynamic stats based on API data
	const stats = $derived([
		{
			title: 'Profit',
			value: formatRupiah(data.stats?.profit || 0),
			subtitle: `${getPercentageChange()} from last month`
		},
		{
			title: 'Revenue',
			value: formatRupiah(data.stats?.revenue || 0),
			subtitle: `${getPercentageChange()} from last month`
		},
		{
			title: 'Orders',
			value: formatNumber(data.stats?.orders || 0),
			subtitle: `${getPercentageChange()} from last month`
		},
		{
			title: 'Conversations',
			value: formatNumber(data.stats?.conversations || 0),
			subtitle: `${getPercentageChange()} from last month`
		}
	]);

	// Chart data for revenue trends
	const chartLabels = [
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
	const realRevenueData = [45, 52, 48, 61, 55, 67, 72, 78, 69, 75, 82, 89]; // in millions
	const predictionRevenueData = [50, 58, 54, 68, 62, 75, 80, 87, 78, 85, 92, 98]; // in millions
</script>

<h1 class="mb-4 text-xl font-semibold">Dashboard Overview</h1>

<!-- Stat Cards -->
<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
	{#each stats as s}
		<Card title={s.title} value={s.value} subtitle={s.subtitle}>
			{#snippet right()}
				<!-- [ID] Sparklines kecil di sisi kanan kartu
             [EN] Tiny sparkline at the card's right side -->
				<svg viewBox="0 0 120 32" width="100" height="26" class="opacity-70">
					<polyline
						fill="none"
						stroke="currentColor"
						stroke-width="3"
						points="0,24 15,20 30,22 45,16 60,20 75,12 90,18 105,10 120,14"
					/>
				</svg>
			{/snippet}
		</Card>
	{/each}
</div>

<!-- Revenue Trend Chart -->
<div class="mt-6">
	<Card title="Revenue Trend">
		{#snippet footer()}
			<div class="h-80">
				<DualLineChart
					labels={chartLabels}
					realData={realRevenueData}
					predictionData={predictionRevenueData}
					height={320}
					realColor="#E98621"
					predictionColor="#3b82f6"
				/>
			</div>
		{/snippet}
	</Card>
</div>

<!-- Weather Insights & Product Recommendations -->
<div class="mt-6">
	<WeatherWidget />
</div>

<!-- Recent Activity -->
<div class="mt-6">
	<Card title="Recent Activity">
		{#snippet footer()}
			<ul class="space-y-3">
				<li class="flex items-center gap-3">
					<span class="h-2 w-2 rounded-full bg-green-500"></span>
					<span class="text-sm">New order completed: ORD-1234</span>
					<span class="ml-auto text-xs opacity-60">2 min ago</span>
				</li>
				<li class="flex items-center gap-3">
					<span class="h-2 w-2 rounded-full bg-blue-500"></span>
					<span class="text-sm">Payment received: {formatRupiah(150000)}</span>
					<span class="ml-auto text-xs opacity-60">5 min ago</span>
				</li>
				<li class="flex items-center gap-3">
					<span class="h-2 w-2 rounded-full bg-amber-500"></span>
					<span class="text-sm">New conversation started</span>
					<span class="ml-auto text-xs opacity-60">10 min ago</span>
				</li>
			</ul>
		{/snippet}
	</Card>
</div>
