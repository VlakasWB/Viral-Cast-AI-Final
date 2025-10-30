<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';

	let email = '';
	const idEmail = 'forgot-email';

	function onSubmit(e: Event) {
		e.preventDefault();
		console.log('Send reset link to', email);
	}
</script>

<form class="space-y-4" onsubmit={onSubmit}>
	<div>
		<label for={idEmail} class="mb-1 block text-sm font-medium">Email</label>
		<input
			id={idEmail}
			class="form-input w-full"
			type="email"
			placeholder="you@email.com"
			bind:value={email}
			required
		/>
	</div>

	<Button label="Send reset link" type="submit" color="accent" />
</form>
