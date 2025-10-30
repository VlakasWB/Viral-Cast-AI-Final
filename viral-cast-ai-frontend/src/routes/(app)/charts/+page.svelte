<script lang="ts">
	const items = [
		{
			href: '/charts/line',
			title: 'Line Chart',
			desc: 'Trends over time with smooth line & tooltips.'
		},
		{ href: '/charts/bar', title: 'Bar Chart', desc: 'Compare categories with neat bars.' },
		{ href: '/charts/pie', title: 'Pie Chart', desc: 'Composition breakdown with legend.' }
	];
</script>

<h1 class="mb-4 text-xl font-semibold">Charts</h1>

<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
	{#each items as it}
		<a
			href={it.href}
			class="rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-card)] transition hover:shadow-[var(--shadow-sidebar)]"
			aria-label={it.title}
		>
			<div class="mb-1 text-base font-medium">{it.title}</div>
			<p class="text-sm opacity-70">{it.desc}</p>
		</a>
	{/each}
</div>
