<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';

	let {
		onSubmit
	}: { onSubmit?: (data: { password: string; confirm: string }) => void | Promise<void> } =
		$props();

	let password = $state('');
	let confirm = $state('');
	let busy = $state(false);
	let errors = $state<{ password?: string; confirm?: string }>({});

	function validate() {
		const e: typeof errors = {};
		if ((password ?? '').length < 6) e.password = 'Min 6 characters';
		if (confirm !== password) e.confirm = 'Passwords do not match';
		errors = e;
		return Object.keys(e).length === 0;
	}

	async function submit(ev: Event) {
		ev.preventDefault();
		if (!validate()) return;
		busy = true;
		try {
			await onSubmit?.({ password, confirm });
		} finally {
			busy = false;
		}
	}
</script>

<div
	class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-6"
>
	<h1 class="mb-1 text-xl font-semibold">Reset password</h1>
	<p class="mb-6 text-sm opacity-70">Enter your new password below.</p>

	<form class="space-y-4" onsubmit={submit}>
		<div>
			<label for="new_password" class="mb-1 block text-sm">New password</label>
			<input
				id="new_password"
				type="password"
				bind:value={password}
				placeholder="••••••••"
				required
				class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] bg-transparent px-4 py-2.5 text-sm outline-none"
			/>
			{#if errors.password}<p class="mt-1 text-xs text-red-500">{errors.password}</p>{/if}
		</div>

		<div>
			<label for="confirm_password" class="mb-1 block text-sm">Confirm password</label>
			<input
				id="confirm_password"
				type="password"
				bind:value={confirm}
				placeholder="••••••••"
				required
				class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] bg-transparent px-4 py-2.5 text-sm outline-none"
			/>
			{#if errors.confirm}<p class="mt-1 text-xs text-red-500">{errors.confirm}</p>{/if}
		</div>

		<Button
			label={busy ? 'Updating…' : 'Update password'}
			color="accent"
			type="submit"
			disabled={busy}
		/>
	</form>

	<p class="mt-6 text-center text-sm">
		Back to <a href="/auth/login" class="text-[--accent] hover:underline">Login</a>
	</p>
</div>
