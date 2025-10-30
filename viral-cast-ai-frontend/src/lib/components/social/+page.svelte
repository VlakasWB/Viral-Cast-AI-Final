<script lang="ts">
	import SocialProviders from '$lib/components/auth/SocialProviders.svelte';
</script>

<section class="mx-auto max-w-3xl space-y-8 px-4">
	<h1 class="text-xl font-semibold">Social / OIDC Providers</h1>

	<div class="space-y-3">
		<h2 class="font-medium">Default (grid)</h2>
		<SocialProviders layout="grid" />
	</div>

	<div class="space-y-3">
		<h2 class="font-medium">Subset (stack, full width)</h2>
		<SocialProviders layout="stack" providers={['google', 'github', 'facebook', 'x']} />
	</div>

	<div class="space-y-3">
		<h2 class="font-medium">Enterprise-ish set</h2>
		<SocialProviders
			layout="grid"
			providers={['microsoft', 'apple', 'linkedin', 'gitlab', 'slack', 'discord']}
		/>
	</div>
</section>
