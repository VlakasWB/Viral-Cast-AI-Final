<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import { browser } from '$app/environment';
	import { enhance } from '$app/forms';

	// ---- terima data action via $props (runes)
	type LoginActionForm = { message?: string };
	let { form }: { form?: LoginActionForm } = $props();

	// ---- state form (Svelte 5 runes)
	let email = $state('');
	let password = $state('');
	let remember = $state(false);
	let busy = $state(false);

	// ---- light client-side error handling
	let errors = $state<{ email?: string; password?: string }>({});

	function validate() {
		const e: typeof errors = {};
		if (!email || email.trim().length < 3)
			e.email = 'Please enter a valid username (min 3 characters)';
		if (!password || password.length < 3) e.password = 'Password is required (min 3 characters)';
		errors = e;
		return Object.keys(e).length === 0;
	}
</script>

<!-- Full screen two columns -->
<div class="grid min-h-dvh md:grid-cols-2">
	<!-- LEFT: form - centered; back-link stays at top-left -->
	<section class="flex h-full flex-col px-6 py-10 md:px-14">
		<!-- everything else centered -->
		<div class="grid flex-1 place-items-center">
			<div class="mx-auto w-full max-w-xl">
				<h1 class="text-3xl font-semibold">Welcome back</h1>
				<p class="mt-2 mb-5 opacity-70">Sign in to your account to continue.</p>

				<!-- form -->
				<form
					method="post"
					novalidate
					use:enhance={({ cancel }) => {
						if (!validate()) {
							cancel();
						}
						return async ({ update }) => {
							busy = true;
							try {
								await update();
							} finally {
								busy = false;
							}
						};
					}}
					class="space-y-4"
				>
					<div>
						<label for="email" class="form-label">Username</label>
						<input
							id="email"
							type="text"
							name="email"
							class="form-input"
							placeholder="Enter your username"
							bind:value={email}
							aria-invalid={Boolean(errors.email)}
							required
						/>
						{#if errors.email}<p class="form-error">{errors.email}</p>{/if}
					</div>

					<div>
						<label for="password" class="form-label">Password</label>
						<input
							id="password"
							type="password"
							name="password"
							class="form-input"
							placeholder="Enter your password"
							bind:value={password}
							aria-invalid={Boolean(errors.password)}
							required
						/>
						{#if errors.password}<p class="form-error">{errors.password}</p>{/if}
					</div>

					<div class="flex items-center justify-between">
						<label class="flex items-center gap-2 text-sm">
							<input
								type="checkbox"
								name="remember"
								class="rounded border-[var(--ui-border)]"
								bind:checked={remember}
							/>
							Remember me
						</label>
						<a href="/pages/auth/forgot-password" class="text-sm text-[--accent] hover:underline">
							Forgot password?
						</a>
					</div>

					{#if form?.message}
						<p class="text-sm text-red-600">{form.message}</p>
					{/if}

					<Button
						label={busy ? 'Signing in…' : 'Sign in'}
						color="orange"
						type="submit"
						disabled={busy}
					/>
				</form>

				<!-- <p class="mt-6 text-sm">
					Don't have an account?
					<a href="/pages/auth/register" class="text-[--accent] hover:underline">Create account</a>
				</p> -->
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
				Continue your journey with Viral Cast AI and build amazing things.
			</p>
		</div>
	</aside>
</div>
