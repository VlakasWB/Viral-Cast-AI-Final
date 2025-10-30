<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';

	let {
		onSubmit
	}: {
		onSubmit?: (fd: FormData) => void;
	} = $props();

	let email = $state('');
	let password = $state('');
	let remember = $state(false);

	function submit(e: Event) {
		e.preventDefault();
		const fd = new FormData();
		fd.set('email', email.trim());
		fd.set('password', password);
		fd.set('remember', remember ? '1' : '0');
		onSubmit?.(fd);
	}
</script>

<div
	class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-6"
>
	<h1 class="mb-1 text-xl font-semibold">Sign in</h1>
	<p class="mb-6 text-sm opacity-70">Welcome back! Please login to your account.</p>

	<form class="space-y-4" onsubmit={submit}>
		<div>
			<label class="mb-1 block text-sm" for="email">Email</label>
			<input
				id="email"
				type="email"
				class="form-input"
				bind:value={email}
				placeholder="you@example.com"
				required
			/>
		</div>

		<div>
			<label class="mb-1 block text-sm" for="password">Password</label>
			<input
				id="password"
				type="password"
				class="form-input"
				bind:value={password}
				placeholder="••••••••"
				required
			/>
		</div>

		<div class="flex items-center justify-between">
			<label class="flex items-center gap-2 text-sm" for="remember">
				<input id="remember" type="checkbox" bind:checked={remember} class="rounded border-[var(--ui-border)]" />
				Remember me
			</label>
			<a href="/auth/forgot-password" class="text-sm text-[--accent] hover:underline"
				>Forgot password?</a
			>
		</div>

		<Button label="Sign in" color="accent" type="submit" />
	</form>

	<p class="mt-6 text-center text-sm">
		Don’t have an account?
		<a href="/auth/register" class="text-[--accent] hover:underline">Create one</a>
	</p>
</div>
