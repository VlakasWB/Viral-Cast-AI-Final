import { browser } from '$app/environment';

export type Mode = 'light' | 'dark';

const MODE_KEY = 'mode';
// [REMOVED] THEME_KEY & daftar themes: tidak lagi menggunakan data-theme

export function apply(mode: Mode) {
	if (!browser) return;
	const el = document.documentElement;
	el.dataset.mode = mode;
	el.classList.toggle('dark', mode === 'dark');
}

export function initTheme() {
	if (!browser) return;
	const mode = 'dark';
	apply(mode);
	localStorage.setItem(MODE_KEY, mode);
	try { localStorage.removeItem('theme'); } catch {}
}

export function setMode(mode: Mode) {
	if (!browser) return;
	localStorage.setItem(MODE_KEY, mode);
	apply(mode);
}

// [REMOVED] setTheme: Dukungan theme berbasis data-theme dihapus.

// Shadow helpers: control dataset + persist
export function setShadow(on: boolean) {
    if (!browser) return;
    const el = document.documentElement;
    const value = on ? 'strong' : 'off';
    el.dataset.shadow = value;
    try {
        localStorage.setItem('shadow', value);
    } catch {}
}

export function toggleShadow() {
    if (!browser) return;
    const el = document.documentElement;
    const isOn = el.dataset.shadow === 'strong';
    setShadow(!isOn);
}
