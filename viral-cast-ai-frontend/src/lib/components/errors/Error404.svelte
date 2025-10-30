<script lang="ts">
	export let status = 404;
	export const error: any = undefined; // Mark as const to indicate external reference only
</script>

<section class="mx-auto max-w-lg p-8 text-center">
	<h1 class="text-4xl font-bold">{status}</h1>
	<p class="mt-2 opacity-80">Halaman tidak ditemukan.</p>
	<a href="/" class="mt-6 inline-block rounded-lg border px-4 py-2">Kembali ke beranda</a>
</section>
