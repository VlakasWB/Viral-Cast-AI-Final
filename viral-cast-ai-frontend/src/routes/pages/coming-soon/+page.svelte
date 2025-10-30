<script lang="ts">
	// [ID] Countdown client-only: guard agar aman SSR
	// [EN] Client-only countdown: SSR-safe guard
	import { browser } from '$app/environment';
	import { onMount } from 'svelte';
	import Button from '$lib/components/ui/Button.svelte';

	type TRemain = { days: number; hours: number; minutes: number; seconds: number };
	let title = $state('Something awesome is coming');
	let subtitle = $state('We’re cooking up new features. Stay tuned!');
	let remaining = $state<TRemain>({ days: 0, hours: 0, minutes: 0, seconds: 0 });

	// [ID] Target default: +30 hari; ubah sesuai kebutuhan (ISO string/UTC)
	// [EN] Default target: +30 days; adjust as needed
	const targetAt = new Date(Date.now() + 30 * 24 * 60 * 60 * 1000);

	function tick() {
		const diff = Math.max(0, targetAt.getTime() - Date.now());
		const s = Math.floor(diff / 1000);
		remaining = {
			days: Math.floor(s / 86400),
			hours: Math.floor((s % 86400) / 3600),
			minutes: Math.floor((s % 3600) / 60),
			seconds: s % 60
		};
	}

	onMount(() => {
		if (!browser) return;
		tick();
		const id = setInterval(tick, 1000);
		return () => clearInterval(id);
	});
</script>

<svelte:head>
	<title>Coming Soon</title>
</svelte:head>

<div class="mx-auto max-w-xl text-center">
	<div
		class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-8"
	>
		<h1 class="mb-2 text-xl font-semibold">{title}</h1>
		<p class="mb-6 text-sm opacity-70">{subtitle}</p>

		<!-- [ID] Countdown -->
		<!-- [EN] Countdown -->
		<div class="mx-auto mb-6 grid max-w-md grid-cols-4 gap-2">
			{#each [['Days', remaining.days], ['Hours', remaining.hours], ['Minutes', remaining.minutes], ['Seconds', remaining.seconds]] as item}
				<div class="rounded-[var(--radius-card)] border border-[var(--ui-border)] p-3">
					<div class="text-2xl font-semibold tabular-nums">{item[1]}</div>
					<div class="text-xs opacity-70">{item[0]}</div>
				</div>
			{/each}
		</div>

		<!-- [ID] Email capture dummy (tanpa submit real) -->
		<!-- [EN] Simple email capture (no real submit) -->
		<form
			class="mx-auto mb-4 flex max-w-md gap-2"
			onsubmit={(e) => { e.preventDefault(); /* no-op */ }}
		>
			<input
				type="email"
				required
				placeholder="you@example.com"
				class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] bg-transparent px-4 py-2.5 text-sm outline-none"
			/>
			<Button color="accent" label="Notify me" />
		</form>

		<p class="text-xs opacity-60">
			[ID] Kami hanya akan mengirim email saat fitur siap. [EN] We’ll only email you once the
			feature is ready.
		</p>
	</div>
</div>
