import { writable } from 'svelte/store';
import { apibase } from '$lib/config';

export interface User {
	id: string;
	email: string;
	role: string;
}

export const user = writable<User | null>(null);
export const loading = writable(true);

/** device copies of the account, used while the backend cannot be reached */
export const saved = { user: 'cwdb:user', profile: 'cwdb:profile' };

function lastuser(): User | null {
	try {
		return JSON.parse(localStorage.getItem(saved.user) ?? 'null');
	} catch {
		return null;
	}
}

/** forgets the account on this device */
function forget() {
	try {
		localStorage.removeItem(saved.user);
		localStorage.removeItem(saved.profile);
	} catch {}
}

/** shows the account last seen on this device at once, then checks it; only a 401 logs out */
export async function fetchUser() {
	const last = lastuser();
	user.set(last);
	loading.set(!last);
	try {
		const res = await fetch(`${apibase}/api/auth/me`, {
			credentials: 'include'
		});
		if (res.ok) {
			const data = await res.json();
			user.set(data);
			localStorage.setItem(saved.user, JSON.stringify(data));
		} else if (res.status === 401) {
			user.set(null);
			forget();
		}
	} catch {
		// offline: keep the account from this device
	} finally {
		loading.set(false);
	}
}

export function login() {
	if (import.meta.env.VITE_USE_MOCK_AUTH === 'true') {
		const email = import.meta.env.VITE_MOCK_EMAIL || 'student@iiit.ac.in';
		window.location.href = `${apibase}/api/auth/mock/login?email=${encodeURIComponent(email)}`;
	} else {
		window.location.href = `${apibase}/api/auth/login`;
	}
}

export function logout() {
	forget();
	user.set(null);
	loading.set(true);
	window.location.href = `${apibase}/api/auth/logout`;
}

/** display name from the part of an email before the @ */
export const nameof = (email: string) =>
	email
		.split('@')[0]
		.split(/[._-]/)
		.map((p) => p.charAt(0).toUpperCase() + p.slice(1).toLowerCase())
		.join(' ');
