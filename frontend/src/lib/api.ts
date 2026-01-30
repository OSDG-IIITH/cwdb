const API_BASE = 'http://localhost:3000';

export interface SearchHit {
    id: number;
    filename: string;
    path: string;
    owner: string;
    repo: string;
    branch: string;
    tags: {
        course?: string;
        shortCourse?: string;
        type?: string;
    };
}

export interface SearchResponse {
    hits: SearchHit[];
}

export async function searchResources(query: string): Promise<SearchHit[]> {
    try {
        const res = await fetch(`${API_BASE}/api/search?q=${encodeURIComponent(query)}`, {
            credentials: 'include',
        });
        if (!res.ok) {
            return [];
        }
        const data: SearchResponse = await res.json();
        return data.hits;
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

export interface Resource {
    id: number;
    source_id: number;
    owner: string;
    repo: string;
    branch: string;
    file_path: string;
    title: string;
    like_count: number;
}

export async function listAllResources(): Promise<Resource[]> {
    try {
        const res = await fetch(`${API_BASE}/api/resources`, {
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