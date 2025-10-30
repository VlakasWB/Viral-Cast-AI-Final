<script lang="ts">
	export let status: number;
	export let error: any;

	import E404 from '$lib/components/errors/Error404.svelte';
	import E500 from '$lib/components/errors/Error500.svelte';
	import E503 from '$lib/components/errors/Error503.svelte';

	const View = status === 404 ? E404 : status === 503 ? E503 : E500; // default ke 500 utk selain 404/503
</script>

<View {status} {error} />
