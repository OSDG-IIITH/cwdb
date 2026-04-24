import { writable } from 'svelte/store';
import { apibase } from '$lib/config';

export interface User {
	id: string;
	email: string;
	role: string;
}

export const user = writable<User | null>(null);
export const loading = writable(true);

export async function fetchUser() {
	loading.set(true);
	try {
		const res = await fetch(`${apibase}/api/auth/me`, {
			credentials: 'include'
		});
		if (res.ok) {
			const data = await res.json();
			user.set(data);
		} else {
			user.set(null);
		}
	} catch {
		user.set(null);
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
	user.set(null);
	loading.set(true);
	window.location.href = `${apibase}/api/auth/logout`;
}
