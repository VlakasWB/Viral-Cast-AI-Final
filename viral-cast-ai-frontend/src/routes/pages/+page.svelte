<script lang="ts">
	// daftar item pages
	type Item = { href: string; title: string; desc: string };

	const items: Item[] = [
		{
			href: '/pages/404',
			title: '404 Not Found',
			desc: 'Halaman tidak ditemukan. Tampilkan pesan & tombol kembali ke beranda.'
		},
		{
			href: '/pages/500',
			title: '500 Server Error',
			desc: 'Terjadi kesalahan pada server. Cocok untuk fallback error umum.'
		},
		{
			href: '/pages/503',
			title: '503 Service Unavailable',
			desc: 'Mode down/over-capacity. Sertakan info estimasi perbaikan.'
		},
		{
			href: '/pages/coming-soon',
			title: 'Coming Soon',
			desc: 'Teaser halaman/fitur yang belum rilis + form subscribe opsional.'
		},
		{
			href: '/pages/maintenance',
			title: 'Maintenance',
			desc: 'Situs sedang perawatan. Tampilkan waktu & kontak dukungan.'
		},
		{
			href: '/pages/login',
			title: 'Login (Full)',
			desc: 'Layout auth full-page 2 kolom + Social.'
		},
		{
			href: '/pages/register',
			title: 'Register (Full)',
			desc: 'Halaman pendaftaran full-page selaras dengan login.'
		}
	];
</script>

<section class="space-y-4">
	<h1 class="text-xl font-semibold">Pages</h1>
	<p class="opacity-70">
		Kumpulan contoh halaman status/otentikasi sistem. Klik salah satu untuk melihat demo.
	</p>

	<!-- grid responsive: 1 / 2 / 3 kolom -->
	<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
		{#each items as it}
			<a
				href={it.href}
				class="rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)]
               p-4 shadow-[var(--shadow-card)] transition hover:shadow-[var(--shadow-sidebar)] focus:ring-2
               focus:ring-[--nav-active-ring] focus:ring-offset-2 focus:outline-none"
				aria-label={it.title}
			>
				<div class="mb-1 text-base font-medium">{it.title}</div>
				<p class="text-sm opacity-70">{it.desc}</p>
			</a>
		{/each}
	</div>
</section>
