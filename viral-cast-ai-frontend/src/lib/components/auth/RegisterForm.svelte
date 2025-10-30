<script lang="ts">
	// [ID] Konsisten dengan login: pakai komponen Button internal
	// [EN] Match login: reuse internal Button component
	import Button from '$lib/components/ui/Button.svelte';

	let {
		onSubmit,
		pending = false,
		title = 'Create your account',
		subtitle = 'Start your journey in a minute'
	}: {
		onSubmit?: (data: {
			name: string;
			email: string;
			password: string;
			confirm: string;
			agree: boolean;
		}) => void | Promise<void>;
		pending?: boolean;
		title?: string;
		subtitle?: string;
	} = $props();

	let name = $state('');
	let email = $state('');
	let password = $state('');
	let confirm = $state('');
	let agree = $state(true);
	let busy = $state(false);

	let errors = $state<{
		name?: string;
		email?: string;
		password?: string;
		confirm?: string;
		agree?: string;
	}>({});

	// [ID] Validasi sederhana di client; server tetap perlu verifikasi
	// [EN] Simple client-side validation; keep server-side checks
	function validate() {
		const e: typeof errors = {};
		if (!name) e.name = 'Name is required';
		if (!email || !/.+@.+\..+/.test(email)) e.email = 'Please enter a valid email';
		if (!password || password.length < 6) e.password = 'Min 6 characters';
		if (confirm !== password) e.confirm = 'Passwords do not match';
		if (!agree) e.agree = 'You must agree to terms';
		errors = e;
		return Object.keys(e).length === 0;
	}

	async function submit(ev: Event) {
		ev.preventDefault();
		if (!validate()) return;
		busy = true;
		try {
			await onSubmit?.({ name, email, password, confirm, agree });
		} finally {
			busy = false;
		}
	}
</script>

<!-- [ID] Sama seperti login: wrapper card + heading + subheading -->
<!-- [EN] Match login: card wrapper + heading + subheading -->
<div
	class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-6"
>
	<h1 class="mb-1 text-xl font-semibold">{title}</h1>
	<p class="mb-6 text-sm opacity-70">{subtitle}</p>

	<form class="space-y-4" onsubmit={submit} novalidate>
		<div>
			<label class="mb-1 block text-sm" for="name">Name</label>
			<input
				id="name"
				type="text"
				required
				class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] bg-transparent px-4 py-2.5 outline-none"
				placeholder="Your name"
				bind:value={name}
				aria-invalid={Boolean(errors.name)}
			/>
			{#if errors.name}<p class="mt-1 text-xs text-red-500">{errors.name}</p>{/if}
		</div>

		<div>
			<label class="mb-1 block text-sm" for="email">Email</label>
			<input
				id="email"
				type="email"
				required
				class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] bg-transparent px-4 py-2.5 outline-none"
				placeholder="you@example.com"
				bind:value={email}
				aria-invalid={Boolean(errors.email)}
			/>
			{#if errors.email}<p class="mt-1 text-xs text-red-500">{errors.email}</p>{/if}
		</div>

		<div>
			<label class="mb-1 block text-sm" for="password">Password</label>
			<input
				id="password"
				type="password"
				required
				class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] bg-transparent px-4 py-2.5 outline-none"
				placeholder="••••••••"
				bind:value={password}
				aria-invalid={Boolean(errors.password)}
			/>
			{#if errors.password}<p class="mt-1 text-xs text-red-500">{errors.password}</p>{/if}
		</div>

		<div>
			<label class="mb-1 block text-sm" for="confirm">Confirm password</label>
			<input
				id="confirm"
				type="password"
				required
				class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] bg-transparent px-4 py-2.5 outline-none"
				placeholder="••••••••"
				bind:value={confirm}
				aria-invalid={Boolean(errors.confirm)}
			/>
			{#if errors.confirm}<p class="mt-1 text-xs text-red-500">{errors.confirm}</p>{/if}
		</div>

		<div class="flex items-center justify-between">
			<label class="flex items-center gap-2 text-sm" for="agree">
				<input id="agree" type="checkbox" bind:checked={agree} class="rounded border-[var(--ui-border)]" />
				I agree to the
				<a href="/legal/terms" class="text-[--accent] hover:underline">Terms</a>
			</label>
		</div>
		{#if errors.agree}<p class="mt-1 text-xs text-red-500">{errors.agree}</p>{/if}

		<Button
			label={busy || pending ? 'Creating…' : 'Create account'}
			color="accent"
			type="submit"
			disabled={busy || pending}
		/>
	</form>

	<p class="mt-6 text-center text-sm">
		Already have an account?
		<a href="/auth/login" class="text-[--accent] hover:underline">Login</a>
	</p>
</div>
