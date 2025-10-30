// [ID] Store UI: collapse sidebar (persist lokal, aman SSR)
// [EN] UI store: sidebar collapsed state (local persistence, SSR-safe)
import { writable } from 'svelte/store';
import { browser } from '$app/environment';

const initial = browser ? localStorage.getItem('sb-collapsed') === '1' : false;
export const sidebarCollapsed = writable<boolean>(initial);

if (browser) {
	sidebarCollapsed.subscribe((v) => {
		try {
			localStorage.setItem('sb-collapsed', v ? '1' : '0');
			// Opsional: beri data-attr untuk styling global jika perlu
			document.documentElement.dataset.sb = v ? 'collapsed' : 'expanded';
		} catch {}
	});
}
