<script lang="ts">
	// [ID] Komponen input cari; emit onSearch(q) saat Enter ditekan
	// [EN] Search input; emits onSearch(q) when Enter is pressed
	import { t } from '$lib/stores/i18n';

	let {
		placeholder = t('press_enter_to_search'),
		value = '',
		onSearch
	}: { placeholder?: string; value?: string; onSearch?: (q: string) => void } = $props();

	let q = $state(value ?? '');

	// sinkron jika parent mengganti value (mis. navigasi, back/forward)
	// tapi jangan ganggu saat user sedang mengetik
	$effect(() => {
		const newValue = value ?? '';
		// Hanya update jika nilai dari parent berbeda dan bukan dari user input
		if (newValue !== q && document.activeElement?.tagName !== 'INPUT') {
			q = newValue;
		}
	});

	// Handle Enter key press untuk trigger search
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			const trimmedValue = q.trim();
			onSearch?.(trimmedValue);
		}
	}

	// Handle form submit (jika dibungkus dalam form)
	function handleSubmit(event: Event) {
		event.preventDefault();
		const trimmedValue = q.trim();
		onSearch?.(trimmedValue);
	}
</script>

<form onsubmit={handleSubmit} class="relative">
	<div class="relative">
		<input
			type="search"
			{placeholder}
			bind:value={q}
			onkeydown={handleKeydown}
			aria-label="Search"
			class="form-input pl-10 text-sm"
		/>
		<div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
			<svg
				class="h-4 w-4 text-gray-500 dark:text-gray-400"
				aria-hidden="true"
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 20 20"
			>
				<path
					stroke="currentColor"
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z"
				/>
			</svg>
		</div>
	</div>
	<button type="submit" class="sr-only">Search</button>
</form>
