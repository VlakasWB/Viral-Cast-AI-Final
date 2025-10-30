<script lang="ts">
	import { onMount } from 'svelte';
	import { enhance } from '$app/forms';

	let logoutForm: HTMLFormElement;

	onMount(() => {
		// Auto-trigger logout action when page loads
		if (logoutForm) {
			logoutForm.requestSubmit();
		}
	});
</script>

<!-- Hidden form for auto-logout -->
<form bind:this={logoutForm} method="POST" use:enhance style="display: none;">
	<button type="submit">Logout</button>
</form>

<section class="mx-auto w-full max-w-7xl px-4 py-16 text-center">
	<h1 class="text-xl font-semibold">You've been logged out</h1>
	<p class="mt-2 opacity-75">Your session has been cleared.</p>
	<a
		href="/login"
		class="mt-6 inline-block rounded-[var(--radius-pill)] border border-[--ui-border] px-4 py-2"
	>
		Back to login
	</a>
</section>
