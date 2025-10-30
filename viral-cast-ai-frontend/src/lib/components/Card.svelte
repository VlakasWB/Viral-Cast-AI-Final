<!-- src/lib/components/Card.svelte -->
<script lang="ts">
	import type { Snippet } from 'svelte';
	let {
		title = '',
		value = '',
		subtitle = '',
		right,
		footer
	}: {
		title?: string;
		value?: string | number;
		subtitle?: string;
		right?: Snippet;
		footer?: Snippet;
	} = $props();
</script>

<section
	class="rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[--surface] bg-[var(--color-surface)] p-4 md:p-5"
	style="box-shadow: var(--elev-card)"
>
	<div class="flex items-start justify-between gap-3">
		<div>
			{#if title}<h3 class="text-sm/6 opacity-80">{title}</h3>{/if}
			{#if value}<div class="mt-1 text-xl font-semibold">{value}</div>{/if}
			{#if subtitle}<p class="mt-1 text-xs opacity-70">{subtitle}</p>{/if}
		</div>
		{#if right}{@render right()}{/if}
	</div>

	{#if footer}
		<div class="mt-4">
			{@render footer()}
		</div>
	{/if}
</section>
