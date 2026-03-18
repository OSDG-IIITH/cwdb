const API_BASE = import.meta.env.VITE_PUBLIC_API_BASE_URL ?? 'http://localhost:3000';

import { z } from 'zod';

export const ResourceSchema = z.object({
    id: z.number(),
    source_id: z.number(),
    owner: z.string(),
    repo: z.string(),
    branch: z.string(),
    file_path: z.string(),
    title: z.string(),
    type: z.string().nullable().optional(),
    like_count: z.number(),
});

export type Resource = z.infer<typeof ResourceSchema>;

export const SearchResponseSchema = z.object({
    resources: z.array(ResourceSchema),
});

export async function searchResources(query: string): Promise<Resource[]> {
    try {
        const res = await fetch(`${API_BASE}/api/search?q=${encodeURIComponent(query)}`, {
            credentials: 'include',
        });
        if (!res.ok) {
            return [];
        }
        const json = await res.json();
        const data = SearchResponseSchema.parse(json);
        return data.resources;
    } catch (e) {
        console.error("Search failed:", e);
        return [];
    }
}

export interface Source {
    id: number;
    owner: string;
    repo: string;
    branch: string;
    source_status: 'active' | 'error' | 'archived' | 'pending';
    last_synced_at: string | null;
    created_at: string | null;
    created_by: number;
    like_count: number;
    liked: boolean;
}

export async function listSources(filter?: string): Promise<Source[]> {
    try {
        const query = filter ? `?filter=${filter}` : '';
        const res = await fetch(`${API_BASE}/api/sources${query}`, {
            credentials: 'include',
        });
        if (!res.ok) {
            const errorText = await res.text();
            console.error("Fetch sources failed:", res.status, errorText);
            return [];
        }
        const data = await res.json();
        return data.sources || [];
    } catch (e) {
        console.error("Fetch sources failed:", e);
        return [];
    }
}

export async function createSource(owner: string, repo: string, branch?: string): Promise<Source | null> {
    try {
        const res = await fetch(`${API_BASE}/api/sources`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ owner, repo, branch }),
            credentials: 'include',
        });
        if (!res.ok) {
            const errorText = await res.text();
            console.error("Create source failed:", res.status, errorText);
            return null;
        }
        return await res.json();
    } catch (e) {
        console.error("Create source failed:", e);
        return null;
    }
}

export async function refreshSource(sourceId: number): Promise<boolean> {
    try {
        const res = await fetch(`${API_BASE}/api/sources/${sourceId}/sync`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'include',
        });
        return res.ok;
    } catch (e) {
        console.error("Sync source failed:", e);
        return false;
    }
}

export async function toggleSourceLike(sourceId: number): Promise<{ liked: boolean; like_count: number } | null> {
    try {
        const res = await fetch(`${API_BASE}/api/sources/${sourceId}/like`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'include',
        });
        if (!res.ok) {
            console.error("Toggle source like failed:", res.status);
            return null;
        }
        return await res.json();
    } catch (e) {
        console.error("Toggle source like failed:", e);
        return null;
    }
}



export async function listAllResources(owner?: string, repo?: string): Promise<Resource[]> {
    try {
        let url = `${API_BASE}/api/resources`;
        if (owner && repo) {
            url += `?owner=${encodeURIComponent(owner)}&repo=${encodeURIComponent(repo)}`;
        }
        const res = await fetch(url, {
            credentials: 'include',
        });
        if (!res.ok) {
            console.error("Fetch resources failed:", res.status);
            return [];
        }
        const data = await res.json();
        return data.resources || [];
    } catch (e) {
        console.error("Fetch resources failed:", e);
        return [];
    }
}