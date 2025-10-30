<!-- src/lib/components/auth/SocialProviders.svelte -->
<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { ProviderKey } from '$lib/types/auth';

	// ✅ Event map: komponen ini meng-emmit event 'select'
	const dispatch = createEventDispatcher<{ select: ProviderKey }>();

	let {
		providers = ['google', 'github', 'facebook', 'x', 'apple', 'microsoft'] as ProviderKey[],
		size = 'md' as 'sm' | 'md' | 'lg',
		layout = 'grid' as 'stack' | 'grid',
		fullWidth = true,
		startPath = '/oauth', // dipakai kalau mode = 'link'
		// ⬇️ new: cara kerja klik
		mode = 'link' as 'link' | 'emit', // 'link' => navigate pakai href, 'emit' => hanya dispatch event
		class: className = ''
	} = $props();

	const META: Record<
		ProviderKey,
		{
			label: string;
			bg: string;
			text?: string;
			border?: string;
		}
	> = {
		google: { label: 'Google', bg: '#4285F4' },
		github: { label: 'GitHub', bg: '#24292F' },
		facebook: { label: 'Facebook', bg: '#1877F2' },
		x: { label: 'X (Twitter)', bg: '#000000' },
		apple: { label: 'Apple', bg: '#000000' },
		microsoft: { label: 'Microsoft', bg: '#2F2F2F' },
		linkedin: { label: 'LinkedIn', bg: '#0A66C2' },
		gitlab: { label: 'GitLab', bg: '#FC6D26' },
		discord: { label: 'Discord', bg: '#5865F2' },
		slack: { label: 'Slack', bg: '#611F69' },
		twitch: { label: 'Twitch', bg: '#9146FF' },
		reddit: { label: 'Reddit', bg: '#FF4500' },
		spotify: { label: 'Spotify', bg: '#1DB954' },
		amazon: { label: 'Amazon', bg: '#FF9900' },
		yahoo: { label: 'Yahoo', bg: '#5F01D1' }
	};

	const sizeCls = $derived.by(() => {
		switch (size) {
			case 'sm':
				return 'text-xs px-3 py-2';
			case 'lg':
				return 'text-base px-5 py-3';
			default:
				return 'text-sm px-4 py-2.5';
		}
	});

	const wrapCls = $derived(
		[layout === 'grid' ? 'grid gap-2 sm:grid-cols-2' : 'flex flex-col gap-2', className].join(' ')
	);

	function icon(p: ProviderKey) {
		// Ikon minimal (inline SVG, aria-hidden)
		switch (p) {
			case 'google':
				return `
        <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
          <path fill="#EA4335" d="M12 10.2v3.9h5.5c-.24 1.3-1.67 3.7-5.5 3.7-3.33 0-6.05-2.76-6.05-6.15s2.72-6.15 6.05-6.15c1.9 0 3.17.8 3.9 1.5l2.65-2.56C16.8 3.08 14.57 2 12 2 6.98 2 2.9 6.08 2.9 11.1S6.98 20.2 12 20.2c6.15 0 9.1-4.31 9.1-8.68 0-.58-.06-1.02-.14-1.47H12z"/>
        </svg>`;
			case 'github':
				return `
        <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M12 .5a12 12 0 00-3.79 23.4c.6.11.82-.26.82-.58l-.02-2.03c-3.34.73-4.04-1.61-4.04-1.61-.55-1.39-1.34-1.76-1.34-1.76-1.09-.75.08-.74.08-.74 1.21.09 1.85 1.25 1.85 1.25 1.07 1.84 2.8 1.31 3.49 1 .11-.79.42-1.31.76-1.61-2.67-.3-5.47-1.34-5.47-5.95 0-1.31.47-2.38 1.24-3.22-.12-.31-.54-1.57.12-3.27 0 0 1.01-.32 3.3 1.23a11.4 11.4 0 016 0c2.29-1.55 3.3-1.23 3.3-1.23.66 1.7.24 2.96.12 3.27.77.84 1.24 1.9 1.24 3.22 0 4.62-2.8 5.64-5.48 5.94.43.37.82 1.1.82 2.22l-.02 3.29c0 .32.22.69.82.58A12 12 0 0012 .5z"/>
        </svg>`;
			case 'facebook':
				return `
        <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M22 12.06C22 6.48 17.52 2 11.94 2S2 6.48 2 12.06C2 17.08 5.66 21.21 10.44 22v-7.01H7.9v-2.93h2.54V9.41c0-2.5 1.49-3.88 3.76-3.88 1.09 0 2.23.2 2.23.2v2.45h-1.26c-1.24 0-1.63.77-1.63 1.56v1.87h2.78l-.44 2.93h-2.34V22C18.34 21.21 22 17.08 22 12.06z"/>
        </svg>`;
			case 'x': // Twitter X
				return `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M18.9 2H21l-6.7 7.7L22 22h-5.6l-4.4-5.8L6.9 22H3.8l7.3-8.4L2 2h5.8l4 5.3L18.9 2zm-2 17h1.6L8.7 4H7.1l9.8 15z"/>
        </svg>`;
			case 'apple':
				return `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M16.36 1.64a4.2 4.2 0 01-1.02 3.22c-.87 1.03-2.02 1.62-3.23 1.52a4.06 4.06 0 011.06-3.1 4.5 4.5 0 013.19-1.64zM20.64 17.9c-.6 1.37-1.33 2.73-2.35 2.75-1.02.02-1.35-.65-2.52-.65-1.17 0-1.54.63-2.52.67-1.01.04-1.78-1.49-2.4-2.85-1.3-2.9-2.29-8.2.98-8.4 1.1-.09 2.03.76 2.52.76.48 0 1.73-.94 2.89-.8 1.24.2 2.25 1.2 2.67 2.56-2.34 1.4-1.96 4.59.73 5.76z"/>
        </svg>`;
			case 'microsoft':
				return `<svg viewBox="0 0 23 23" width="16" height="16" aria-hidden="true">
          <path fill="#F35325" d="M1 1h10v10H1z"/><path fill="#81BC06" d="M12 1h10v10H12z"/>
          <path fill="#05A6F0" d="M1 12h10v10H1z"/><path fill="#FFBA08" d="M12 12h10v10H12z"/>
        </svg>`;
			case 'linkedin':
				return `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M4.98 3.5C4.98 4.88 3.87 6 2.5 6S0 4.88 0 3.5 1.12 1 2.5 1s2.48 1.12 2.48 2.5zM.5 8.5h4V23h-4V8.5zM8 8.5h3.8v2h.05c.52-.98 1.8-2 3.7-2 3.95 0 4.68 2.6 4.68 6V23h-4v-5.5c0-1.32-.02-3.02-1.84-3.02-1.85 0-2.13 1.44-2.13 2.92V23H8V8.5z"/>
        </svg>`;
			case 'gitlab':
				return `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M22.65 13.4l-2.1-6.48a.9.9 0 00-1.7-.07L16.7 12H7.3L5.15 6.86a.9.9 0 00-1.7.07L1.35 13.4a1.2 1.2 0 00.43 1.32l9.76 7.24a1.2 1.2 0 001.43 0l9.76-7.24a1.2 1.2 0 00.43-1.32z"/>
        </svg>`;
			case 'discord':
				return `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M20.3 4.4A18 18 0 0015.9 3l-.2.4c2 .5 3 1.2 4 2.1-1.7-.9-3.4-1.6-5.6-1.6S9.1 4.6 7.4 5.5c.9-.9 2-1.6 4-2.1L11.2 3c-2 .2-3.7.7-4.4 1.4C3 7.3 2 11 2.3 14.6c1.8 1.3 3.6 2.1 5.3 2.5l.8-1.3c-1.4-.5-2.6-1.2-3.7-2.1.9.7 2.4 1.6 5 2 .5.1 1 .2 1.5.2s1 0 1.5-.2c2.6-.4 4.1-1.3 5-2-.9.8-2.1 1.5-3.6 2l.7 1.3c1.7-.4 3.6-1.2 5.3-2.5.3-3.6-.7-7.3-3.1-10.2z"/>
        </svg>`;
			case 'slack':
				return `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M5.1 15.7a2 2 0 11-2 2h2v-2zm1 0h3v2h-3a2 2 0 110-2zm12.8 0a2 2 0 11-2 2h2v-2zm-4.9 0h3v2h-3v-2zM8.1 9.3H5.1a2 2 0 112-2v2zm1 0v-2h3v2h-3zm8 0h-3v-2h3a2 2 0 110 2zm-4 0h-2v-2h2v2zM9.1 11.5v1.9H7.2a1.9 1.9 0 110-3.8h1.9v1.9zm5.8 0h1.9a1.9 1.9 0 110 3.8H15v-1.9h-.1v-1.9z"/>
        </svg>`;
			case 'twitch':
				return `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M3 2l-1 3v14h5v3h3l3-3h4l5-5V2H3zm18 9l-3 3h-5l-3 3v-3H6V4h15v7z"/>
        </svg>`;
			case 'reddit':
				return `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M22 12.3c0-1-.8-1.9-1.8-1.9-.5 0-1 .2-1.3.5A8.7 8.7 0 0012 9.7c-2 0-3.8.7-5 1.2-.4-.3-.9-.5-1.4-.5C4.6 10.4 4 11.3 4 12.3s.7 1.8 1.7 1.8c.3 0 .6-.1.9-.3A7.37 7.37 0 0012 15.5c1.9 0 3.6-.6 4.9-1.4.3.2.6.3.9.3 1 0 1.8-.9 1.8-1.9zM9.5 13a1.3 1.3 0 11-2.6 0c0-.7.6-1.3 1.3-1.3.7 0 1.3.6 1.3 1.3zm7.6 0a1.3 1.3 0 11-2.6 0c0-.7.6-1.3 1.3-1.3.7 0 1.3.6 1.3 1.3z"/>
        </svg>`;
			case 'spotify':
				return `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M12 1.8A10.2 10.2 0 112 12 10.2 10.2 0 0112 1.8zm4.6 14.9a.8.8 0 00-1.1-.3c-3 1.8-6.8 1.1-8.9 0a.8.8 0 10-.8 1.4c2.5 1.5 6.8 2.2 10.4 0 .4-.2.5-.7.4-1.1zm1.6-3.2a1 1 0 00-1.4-.3c-3.4 2-8.7 1.6-11.6 0a1 1 0 10-1 1.8c3.4 1.8 9.3 2.2 13.3 0 .5-.2.7-.8.4-1.5zm.1-3.5a1.2 1.2 0 00-1.6-.4c-3.9 2.3-10.3 2-13.8 0a1.2 1.2 0 10-1.2 2.1c4.1 2.3 11.6 2.7 16.2 0 .6-.3.9-1 .4-1.7z"/>
        </svg>`;
			case 'amazon':
				return `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M19.1 16.9c-.7.5-4.3 2.4-7.6 2.4-3.2 0-6-1.2-6.3-1.4-.4-.2-.6.1-.3.3.7.6 3.6 2.4 6.5 2.4 3.1 0 6.6-1.3 7.9-2.2.3-.2.1-.6-.2-.5zM7.4 15.4c-.3-.1-.5.3-.2.5 1 .6 2.8 1.1 4.5 1.1 1.8 0 3.7-.4 4.9-1.2.3-.2.1-.6-.2-.5-1.3.3-2.6.9-4.7.9-1.9 0-3.2-.5-4.3-.8z"/>
        </svg>`;
			case 'yahoo':
				return `<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
          <path d="M14.3 4l-3.1 6.6L8 4H4l5.2 10.7V20h3.6v-5.3L18.9 4z"/>
        </svg>`;
		}
	}
	function handleClick(p: ProviderKey, e: MouseEvent) {
		// selalu emit event
		dispatch('select', p);
		// jika hanya ingin emit (tanpa pindah halaman), cegah default anchor
		if (mode === 'emit') e.preventDefault();
	}
</script>

<div class={wrapCls}>
	{#each providers as p}
		{@const meta = META[p]}
		<a
			href={`${startPath}/${p}`}
			onclick={(e) => handleClick(p, e)}
			class={`inline-flex items-center justify-center gap-2 rounded-[var(--radius-pill)]
              text-white ring-1 ring-white/10 transition hover:brightness-105 focus-visible:ring-2
              focus-visible:ring-white/40 focus-visible:outline-none active:brightness-95
              ${fullWidth ? 'w-full' : ''} ${sizeCls}`}
			style={`background:${meta.bg}`}
			aria-label={`Continue with ${meta.label}`}
		>
			{@html icon(p)}
			<span class="whitespace-nowrap">Continue with {meta.label}</span>
		</a>
	{/each}
</div>
