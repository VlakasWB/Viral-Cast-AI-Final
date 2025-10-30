<script lang="ts">
	export let status = 500;
	export let error: any;

	import { locale, t } from '$lib/stores/i18n';

	let errorTitle = '';
	let errorDesc = '';
	let detailsLabel = '';
	let tryAgainLabel = '';
	let goHomeLabel = '';

	$: {
		const __ = $locale; // trigger reactive re-render on locale change
		errorTitle = t('error_500_title');
		errorDesc = t('error_500_desc');
		detailsLabel = t('error_details');
		tryAgainLabel = t('try_again');
		goHomeLabel = t('go_home');
	}

	// Log error for debugging (only when error exists)
	if (error) console.error('500 Error:', error);
</script>

<section class="mx-auto max-w-lg p-8 text-center">
	<div class="mb-6">
		<div class="mb-4 text-6xl">⚠️</div>
		<h1 class="text-4xl font-bold text-red-600 dark:text-red-400">{status}</h1>
		<h2 class="mt-2 mb-4 text-xl font-semibold">{errorTitle}</h2>
		<p class="mb-6 text-gray-600 dark:text-gray-400">
			{errorDesc}
		</p>

		{#if error?.message}
			<details class="mb-6 rounded-lg bg-gray-100 p-4 text-left dark:bg-gray-800">
				<summary class="cursor-pointer font-medium">{detailsLabel}</summary>
				<pre class="mt-2 overflow-auto text-sm text-red-600 dark:text-red-400">{error.message}</pre>
			</details>
		{/if}
	</div>

	<div class="flex justify-center gap-3">
		<button
			onclick={() => window.location.reload()}
			class="rounded-lg bg-blue-600 px-6 py-2 text-white transition-colors hover:bg-blue-700"
		>
			{tryAgainLabel}
		</button>
		<a
			href="/"
			class="rounded-lg border border-gray-300 px-6 py-2 transition-colors hover:bg-gray-50 dark:border-gray-600 dark:hover:bg-gray-800"
		>
			{goHomeLabel}
		</a>
	</div>
</section>
