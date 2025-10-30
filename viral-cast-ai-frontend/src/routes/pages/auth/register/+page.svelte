<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import { browser } from '$app/environment';

	// ---- state form
	let name = $state('');
	let email = $state('');
	let password = $state('');
	let confirm = $state('');
	let agree = $state(true);
	let busy = $state(false);

	// ---- error ringan client
	let errors = $state<{
		name?: string;
		email?: string;
		password?: string;
		confirm?: string;
		agree?: string;
	}>({});

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
			// TODO: kirim ke action/API
			console.log({ name, email, password, confirm, agree });
		} finally {
			busy = false;
		}
	}

	// ---- social (contoh redirect / emit sesuai kebutuhanmu)
	type Provider = 'google' | 'github';
	function signInWith(provider: Provider) {
		if (browser) {
			// location.href = `/auth/oauth/${provider}`;
			console.log('oauth:', provider);
		}
	}

	const inputCls = undefined;
</script>

<!-- Full screen two columns -->
<div class="grid min-h-dvh md:grid-cols-2">
	<!-- LEFT: form - centered; back-link stays at top-left -->
	<section class="flex h-full flex-col px-6 py-10 md:px-14">
		<!-- back link -->
		<a
			href="/"
			class="mb-8 inline-flex items-center gap-2 self-start text-sm opacity-70 hover:opacity-100"
		>
			<svg
				viewBox="0 0 24 24"
				width="16"
				height="16"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<path d="M15 18l-6-6 6-6" />
			</svg>
			<span>Back to dashboard</span>
		</a>

		<!-- everything else centered -->
		<div class="grid flex-1 place-items-center">
			<div class="mx-auto w-full max-w-xl">
				<h1 class="text-3xl font-semibold">Create an account</h1>
				<p class="mt-2 opacity-70">Start your journey with us.</p>

				<!-- form -->
				<form class="space-y-4" onsubmit={submit} novalidate>
					<div>
						<label class="mb-1 block text-sm" for="name">Full name</label>
						<input
							type="text"
							required
							class="form-input"
							placeholder="Your name"
							id="name"
							bind:value={name}
							aria-invalid={Boolean(errors.name)}
						/>
						{#if errors.name}<p class="mt-1 text-xs text-red-500">{errors.name}</p>{/if}
					</div>

					<div>
						<label class="mb-1 block text-sm" for="email">Email</label>
						<input
							type="email"
							required
							class="form-input"
							placeholder="you@example.com"
							id="email"
							bind:value={email}
							aria-invalid={Boolean(errors.email)}
						/>
						{#if errors.email}<p class="mt-1 text-xs text-red-500">{errors.email}</p>{/if}
					</div>

					<div class="grid gap-4 md:grid-cols-2">
						<div>
							<label class="mb-1 block text-sm" for="password">Password</label>
							<input
								type="password"
								required
								class="form-input"
								placeholder="••••••••"
								id="password"
								bind:value={password}
								aria-invalid={Boolean(errors.password)}
							/>
							{#if errors.password}<p class="mt-1 text-xs text-red-500">{errors.password}</p>{/if}
						</div>

						<div>
							<label class="mb-1 block text-sm" for="confirm">Confirm</label>
							<input
								type="password"
								required
								class="form-input"
								placeholder="••••••••"
								id="confirm"
								bind:value={confirm}
								aria-invalid={Boolean(errors.confirm)}
							/>
							{#if errors.confirm}<p class="mt-1 text-xs text-red-500">{errors.confirm}</p>{/if}
						</div>
					</div>

					<label class="mt-2 flex items-center gap-2 text-sm">
						<input type="checkbox" class="rounded border-[var(--ui-border)]" bind:checked={agree} />
						I agree to the&nbsp;<a href="/legal/terms" class="text-[--accent] hover:underline"
							>Terms</a
						>&nbsp;&amp;&nbsp;
						<a href="/legal/privacy" class="text-[--accent] hover:underline">Privacy</a>
					</label>
					{#if errors.agree}<p class="mt-1 text-xs text-red-500">{errors.agree}</p>{/if}

					<Button
						label={busy ? 'Creating…' : 'Create account'}
						color="grape"
						type="submit"
						disabled={busy}
					/>
				</form>

				<p class="mt-6 text-sm">
					Already have an account?
					<a href="/pages/auth/login" class="text-[--accent] hover:underline">Sign in</a>
				</p>
			</div>
		</div>
	</section>

	<!-- RIGHT: brand/hero -->
	<aside
		class="relative hidden items-center justify-center bg-[#121A4A] p-10 text-center text-white md:flex"
	>
		<div
			class="absolute inset-0 opacity-20"
			style="background-image: radial-gradient(white 1px, transparent 1px); background-size: 22px 22px;"
		></div>
		<div class="relative z-10">
			<div class="mx-auto mb-6 grid h-14 w-14 place-items-center rounded-2xl bg-white/10">
				<span class="text-lg font-bold">VC</span>
			</div>
			<h2 class="text-3xl font-semibold">Viral Cast AI</h2>
			<p class="mt-2 text-sm text-white/80">
				Join the community and build faster with Svelte + Tailwind.
			</p>
		</div>
	</aside>
</div>
