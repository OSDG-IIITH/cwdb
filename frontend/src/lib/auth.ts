import { writable } from 'svelte/store';

export interface User {
    id: number;
    email: string;
    role: string;
}

export const user = writable<User | null>(null);
export const loading = writable(true);

const API_BASE = 'http://localhost:3000';

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
    await fetch(`${API_BASE}/api/auth/logout`, {
        method: 'POST',
        credentials: 'include',
    });
    user.set(null);
    window.location.reload();
}
