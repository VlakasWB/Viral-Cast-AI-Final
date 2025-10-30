<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';

	let { onSubmit }: { onSubmit?: (email: string) => void | Promise<void> } = $props();
	let email = $state('');
	let busy = $state(false);
	let error = $state<string | null>(null);

	async function submit(e: Event) {
		e.preventDefault();
		error = !/.+@.+\..+/.test(email) ? 'Please enter a valid email' : null;
		if (error) return;
		busy = true;
		try {
			await onSubmit?.(email.trim());
		} finally {
			busy = false;
		}
	}
</script>

<div
	class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-6"
>
	<h1 class="mb-1 text-xl font-semibold">Forgot password</h1>
	<p class="mb-6 text-sm opacity-70">We’ll send a reset link to your email.</p>

	<form class="space-y-4" onsubmit={submit}>
		<div>
			<label for="email" class="mb-1 block text-sm">Email</label>
			<input
				id="email"
				type="email"
				required
				bind:value={email}
				placeholder="you@example.com"
				class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] bg-transparent px-4 py-2.5 text-sm outline-none"
			/>
			{#if error}<p class="mt-1 text-xs text-red-500">{error}</p>{/if}
		</div>

		<Button
			label={busy ? 'Sending…' : 'Send reset link'}
			color="accent"
			type="submit"
			disabled={busy}
		/>
	</form>

	<p class="mt-6 text-center text-sm">
		Remember your password? <a href="/auth/login" class="text-[--accent] hover:underline">Login</a>
	</p>
</div>
