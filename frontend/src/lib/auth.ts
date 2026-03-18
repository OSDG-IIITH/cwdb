import { writable } from 'svelte/store';

export interface User {
    id: number;
    email: string;
    role: string;
}

export const user = writable<User | null>(null);
export const loading = writable(true);

const API_BASE = import.meta.env.VITE_PUBLIC_API_BASE_URL ?? 'http://localhost:3000';

export async function fetchUser() {
    loading.set(true);
    try {
        const res = await fetch(`${API_BASE}/api/auth/me`, {
            credentials: 'include',
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
    window.location.href = `${API_BASE}/api/auth/login`;
}

export async function logout() {
    loading.set(true);
    try {
        await fetch(`${API_BASE}/api/auth/logout`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'include',
        });
    } catch (e) {
        // The fetch might throw a CORS error when it follows the backend's 302 redirect
        // to the SvelteKit frontend (which lacks cross-origin headers).
        // Since the backend already cleared the session cookie, we can safely ignore it.
    } finally {
        user.set(null);
        loading.set(false);
        window.location.href = '/';
    }
}
