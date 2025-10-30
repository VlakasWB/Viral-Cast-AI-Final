<script lang="ts">
	let {
		title = '',
		value = '',
		note = '',
		trend = 'flat' // 'up' | 'down' | 'flat'
	}: {
		title?: string;
		value?: string;
		note?: string;
		trend?: 'up' | 'down' | 'flat';
	} = $props();
</script>

<div class="shadow-card rounded-2xl border border-[--ui-border] bg-[--color-surface] p-5">
	<div class="flex items-start justify-between">
		<div>
			<div class="text-sm opacity-70">{title}</div>
			<div class="mt-1 text-2xl font-semibold">{value}</div>
			{#if note}<div class="mt-1 text-xs opacity-60">{note}</div>{/if}
		</div>

		<!-- sparkline built-in (opsional) -->
		<svg viewBox="0 0 120 32" width="96" height="26" class="opacity-70">
			{#if trend === 'up'}
				<polyline
					fill="none"
					stroke="currentColor"
					stroke-width="3"
					points="0,24 15,20 30,22 45,16 60,20 75,12 90,18 105,10 120,14"
				/>
			{:else if trend === 'down'}
				<polyline
					fill="none"
					stroke="currentColor"
					stroke-width="3"
					points="0,8 15,10 30,12 45,16 60,18 75,22 90,24 105,26 120,28"
				/>
			{:else}
				<polyline
					fill="none"
					stroke="currentColor"
					stroke-width="3"
					points="0,18 15,18 30,18 45,18 60,18 75,18 90,18 105,18 120,18"
				/>
			{/if}
		</svg>
	</div>
</div>
