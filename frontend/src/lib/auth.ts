import { writable } from 'svelte/store';

export interface User {
    id: string;
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
    if (import.meta.env.VITE_USE_MOCK_AUTH === 'true') {
        const email = import.meta.env.VITE_MOCK_EMAIL || 'student@iiit.ac.in';
        window.location.href = `${API_BASE}/api/auth/mock/login?email=${encodeURIComponent(email)}`;
    } else {
        window.location.href = `${API_BASE}/api/auth/login`;
    }
}

export function logout() {
    user.set(null);
    loading.set(true);
    window.location.href = `${API_BASE}/api/auth/logout`;
}
