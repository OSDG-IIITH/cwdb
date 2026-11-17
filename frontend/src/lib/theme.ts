import { writable } from 'svelte/store';
import { setMode } from 'mode-watcher';

export const variant = writable('default');

/** sets the colour variant class and the light or dark mode it needs */
export function applyvariant(v: string) {
	document.documentElement.classList.remove('mint', 'amethyst');
	if (v === 'mint' || v === 'amethyst') {
		document.documentElement.classList.add(v);
		setMode(v === 'mint' ? 'light' : 'dark');
	}
	localStorage.setItem('theme-variant', v);
	variant.set(v);
}

/** picks one of the four themes */
export function picktheme(key: string) {
	if (key === 'light' || key === 'dark') {
		applyvariant('default');
		setMode(key);
	} else applyvariant(key);
}

/** applies whatever theme is in storage, after synced prefs land */
export function restoretheme() {
	const mode = localStorage.getItem('mode-watcher-mode');
	if (mode === 'light' || mode === 'dark' || mode === 'system') setMode(mode);
	applyvariant(localStorage.getItem('theme-variant') || 'default');
}
