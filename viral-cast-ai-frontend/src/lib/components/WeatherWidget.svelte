<script lang="ts">
	import { onMount } from 'svelte';

	// Weather data interface
	interface WeatherData {
		location: string;
		temperature: number;
		condition: string;
		humidity: number;
		icon: string;
		description: string;
	}

	interface ProductRecommendation {
		name: string;
		reason: string;
		stockSuggestion: string;
		priority: 'high' | 'medium' | 'low';
	}

	// Mock weather data
	let todayWeather: WeatherData = {
		location: 'Jakarta, Indonesia',
		temperature: 28,
		condition: 'Partly Cloudy',
		humidity: 75,
		icon: '⛅',
		description: 'Warm with some clouds'
	};

	let tomorrowWeather = $state<WeatherData>({
		location: 'Jakarta, Indonesia',
		temperature: 24,
		condition: 'Rainy',
		humidity: 85,
		icon: '🌧️',
		description: 'Heavy rain expected'
	});

	// Weather-based product recommendations
	let recommendations = $state<ProductRecommendation[]>([]);

	// State management
	let isExpanded = $state(false);
	let isLoading = $state(false);

	// Generate recommendations based on weather
	function generateRecommendations(
		today: WeatherData,
		tomorrow: WeatherData
	): ProductRecommendation[] {
		const recs: ProductRecommendation[] = [];

		// Check for rainy conditions
		if (
			today.condition.toLowerCase().includes('rain') ||
			tomorrow.condition.toLowerCase().includes('rain')
		) {
			recs.push({
				name: 'Premium Coffee Blend',
				reason: 'Rainy weather increases demand for hot beverages',
				stockSuggestion: 'Increase stock by 40%',
				priority: 'high'
			});
			recs.push({
				name: 'Hot Chocolate',
				reason: 'Comfort drinks are popular during rainy days',
				stockSuggestion: 'Increase stock by 30%',
				priority: 'medium'
			});
			recs.push({
				name: 'Herbal Tea Collection',
				reason: 'Warm beverages help during cold, wet weather',
				stockSuggestion: 'Increase stock by 25%',
				priority: 'medium'
			});
		}

		// Check for hot weather
		if (today.temperature > 30 || tomorrow.temperature > 30) {
			recs.push({
				name: 'Iced Coffee Variants',
				reason: 'Hot weather drives cold beverage sales',
				stockSuggestion: 'Increase stock by 35%',
				priority: 'high'
			});
			recs.push({
				name: 'Fresh Fruit Juices',
				reason: 'Refreshing drinks are in high demand',
				stockSuggestion: 'Increase stock by 30%',
				priority: 'medium'
			});
		}

		// Check for high humidity
		if (today.humidity > 80 || tomorrow.humidity > 80) {
			recs.push({
				name: 'Light Pastries',
				reason: 'High humidity reduces appetite for heavy foods',
				stockSuggestion: 'Focus on lighter options',
				priority: 'low'
			});
		}

		return recs;
	}

	// Toggle widget expansion
	function toggleWidget() {
		isExpanded = !isExpanded;
		if (isExpanded && recommendations.length === 0) {
			loadRecommendations();
		}
	}

	// Load recommendations
	function loadRecommendations() {
		isLoading = true;
		// Simulate API call
		setTimeout(() => {
			recommendations = generateRecommendations(todayWeather, tomorrowWeather);
			isLoading = false;
		}, 800);
	}

	// Format temperature
	function formatTemp(temp: number): string {
		return `${temp}°C`;
	}

	// Get priority color
	function getPriorityColor(priority: string): string {
		switch (priority) {
			case 'high':
				return 'text-red-600 dark:text-red-400';
			case 'medium':
				return 'text-yellow-600 dark:text-yellow-400';
			case 'low':
				return 'text-green-600 dark:text-green-400';
			default:
				return 'text-gray-600 dark:text-gray-400';
		}
	}

	// Simulate weather updates
	onMount(() => {
		const interval = setInterval(() => {
			// Randomly update weather conditions for demo
			const conditions = ['Sunny', 'Partly Cloudy', 'Cloudy', 'Rainy', 'Thunderstorm'];
			const icons = ['☀️', '⛅', '☁️', '🌧️', '⛈️'];
			const randomIndex = Math.floor(Math.random() * conditions.length);

			tomorrowWeather = {
				...tomorrowWeather,
				condition: conditions[randomIndex],
				icon: icons[randomIndex],
				temperature: Math.floor(Math.random() * 10) + 22, // 22-32°C
				humidity: Math.floor(Math.random() * 30) + 60 // 60-90%
			};

			// Update recommendations if widget is expanded
			if (isExpanded) {
				recommendations = generateRecommendations(todayWeather, tomorrowWeather);
			}
		}, 30000); // Update every 30 seconds

		return () => clearInterval(interval);
	});
</script>

<div class="relative">
	<!-- 3D Weather Button -->
	<button
		onclick={toggleWidget}
		class="group relative overflow-hidden rounded-2xl bg-gradient-to-br from-blue-400 via-blue-500 to-blue-600 p-4 shadow-lg transition-all duration-300 hover:scale-105 hover:shadow-xl active:scale-95"
		style="transform-style: preserve-3d;"
	>
		<!-- 3D Effect Background -->
		<div
			class="absolute inset-0 rounded-2xl bg-gradient-to-br from-white/20 to-transparent opacity-50"
		></div>
		<div class="absolute inset-0 rounded-2xl bg-gradient-to-tl from-black/10 to-transparent"></div>

		<!-- Button Content -->
		<div class="relative z-10 flex items-center gap-3 text-white">
			<div class="text-2xl">{todayWeather.icon}</div>
			<div class="text-left">
				<div class="text-sm font-medium">Weather Insights</div>
				<div class="text-xs opacity-90">
					{formatTemp(todayWeather.temperature)} • {todayWeather.condition}
				</div>
			</div>
			<div class="ml-2 transition-transform duration-300 {isExpanded ? 'rotate-180' : ''}">
				<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M19 9l-7 7-7-7"
					/>
				</svg>
			</div>
		</div>

		<!-- Keyboard shortcut indicator -->
		<div class="absolute top-2 right-2 rounded bg-white/20 px-1 text-xs opacity-60">P</div>
	</button>

	<!-- Expanded Weather Widget -->
	{#if isExpanded}
		<div
			class="absolute top-full right-0 left-0 z-50 mt-2 overflow-hidden rounded-xl border border-gray-200 bg-white shadow-2xl dark:border-gray-700 dark:bg-gray-800"
		>
			<!-- Weather Section -->
			<div
				class="bg-gradient-to-r from-blue-50 to-indigo-50 p-6 dark:from-gray-800 dark:to-gray-700"
			>
				<h3 class="mb-4 text-lg font-semibold text-gray-900 dark:text-white">Weather Forecast</h3>

				<div class="grid grid-cols-2 gap-4">
					<!-- Today's Weather -->
					<div class="rounded-lg bg-white p-4 shadow-sm dark:bg-gray-800">
						<div class="mb-1 text-sm text-gray-500 dark:text-gray-400">Today</div>
						<div class="flex items-center gap-3">
							<div class="text-3xl">{todayWeather.icon}</div>
							<div>
								<div class="text-xl font-bold text-gray-900 dark:text-white">
									{formatTemp(todayWeather.temperature)}
								</div>
								<div class="text-sm text-gray-600 dark:text-gray-300">{todayWeather.condition}</div>
								<div class="text-xs text-gray-500 dark:text-gray-400">
									Humidity: {todayWeather.humidity}%
								</div>
							</div>
						</div>
					</div>

					<!-- Tomorrow's Weather -->
					<div class="rounded-lg bg-white p-4 shadow-sm dark:bg-gray-800">
						<div class="mb-1 text-sm text-gray-500 dark:text-gray-400">Tomorrow</div>
						<div class="flex items-center gap-3">
							<div class="text-3xl">{tomorrowWeather.icon}</div>
							<div>
								<div class="text-xl font-bold text-gray-900 dark:text-white">
									{formatTemp(tomorrowWeather.temperature)}
								</div>
								<div class="text-sm text-gray-600 dark:text-gray-300">
									{tomorrowWeather.condition}
								</div>
								<div class="text-xs text-gray-500 dark:text-gray-400">
									Humidity: {tomorrowWeather.humidity}%
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>

			<!-- Product Recommendations Section -->
			<div class="p-6">
				<div class="mb-4 flex items-center justify-between">
					<h3 class="text-lg font-semibold text-gray-900 dark:text-white">
						Smart Product Recommendations
					</h3>
					{#if isLoading}
						<div
							class="h-4 w-4 animate-spin rounded-full border-2 border-blue-500 border-t-transparent"
						></div>
					{/if}
				</div>

				{#if isLoading}
					<div class="space-y-3">
						{#each Array(3) as _}
							<div class="h-16 animate-pulse rounded-lg bg-gray-200 dark:bg-gray-700"></div>
						{/each}
					</div>
				{:else if recommendations.length > 0}
					<div class="space-y-3">
						{#each recommendations as rec}
							<div class="rounded-lg border-l-4 border-blue-500 bg-gray-50 p-4 dark:bg-gray-700">
								<div class="flex items-start justify-between">
									<div class="flex-1">
										<div class="font-medium text-gray-900 dark:text-white">{rec.name}</div>
										<div class="mt-1 text-sm text-gray-600 dark:text-gray-300">{rec.reason}</div>
										<div class="mt-2 text-sm font-medium {getPriorityColor(rec.priority)}">
											{rec.stockSuggestion}
										</div>
									</div>
									<div class="ml-3">
										<span
											class="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium {rec.priority ===
											'high'
												? 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
												: rec.priority === 'medium'
													? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
													: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'}"
										>
											{rec.priority}
										</span>
									</div>
								</div>
							</div>
						{/each}
					</div>
				{:else}
					<div class="py-8 text-center text-gray-500 dark:text-gray-400">
						<div class="mb-2 text-4xl">🌤️</div>
						<div class="text-sm">No specific recommendations for current weather conditions</div>
					</div>
				{/if}

				<!-- Weather Insight -->
				{#if tomorrowWeather.condition.toLowerCase().includes('rain')}
					<div
						class="mt-4 rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-800 dark:bg-blue-900/20"
					>
						<div class="flex items-start gap-3">
							<div class="text-xl text-blue-500">💡</div>
							<div>
								<div class="font-medium text-blue-900 dark:text-blue-100">Rainy Season Alert</div>
								<div class="mt-1 text-sm text-blue-700 dark:text-blue-300">
									Extended rainy weather predicted. Consider increasing hot beverage inventory by
									30-40% to meet increased demand.
								</div>
							</div>
						</div>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<!-- Keyboard shortcut handler -->
<svelte:window
	on:keydown={(e) => {
		if (e.key.toLowerCase() === 'p' && !e.ctrlKey && !e.altKey && !e.metaKey) {
			const target = e.target as HTMLElement;
			if (target.tagName !== 'INPUT' && target.tagName !== 'TEXTAREA') {
				e.preventDefault();
				toggleWidget();
			}
		}
	}}
/>

<style>
	/* 3D Button Effects */
	button {
		box-shadow:
			0 4px 8px rgba(0, 0, 0, 0.1),
			0 1px 3px rgba(0, 0, 0, 0.08),
			inset 0 1px 0 rgba(255, 255, 255, 0.2);
	}

	button:hover {
		box-shadow:
			0 8px 16px rgba(0, 0, 0, 0.15),
			0 2px 6px rgba(0, 0, 0, 0.1),
			inset 0 1px 0 rgba(255, 255, 255, 0.3);
	}

	button:active {
		box-shadow:
			0 2px 4px rgba(0, 0, 0, 0.1),
			0 1px 2px rgba(0, 0, 0, 0.08),
			inset 0 1px 0 rgba(255, 255, 255, 0.1);
	}
</style>
