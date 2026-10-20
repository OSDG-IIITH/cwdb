import { z } from 'zod';
import { writable } from 'svelte/store';
import { apibase } from '$lib/config';

export const SourceSchema = z.object({
	id: z.number(),
	owner: z.string(),
	repo: z.string(),
	branch: z.string(),
	source_status: z.string(),
	last_synced_at: z.string().nullable(),
	created_at: z.string().nullable(),
	created_by: z.string(),
	aliases: z.record(z.string(), z.string()).optional().default({})
});

export type Source = z.infer<typeof SourceSchema>;

export const SourcesResponseSchema = z.object({
	sources: z.array(SourceSchema)
});

export async function listSources(filter?: string): Promise<Source[]> {
	try {
		const query = filter ? `?filter=${filter}` : '';
		const res = await fetch(`${apibase}/api/sources${query}`, { credentials: 'include' });
		if (!res.ok) return [];
		const json = await res.json();
		return SourcesResponseSchema.parse({ sources: json.sources ?? [] }).sources;
	} catch (e) {
		console.error('Fetch sources failed:', e);
		return [];
	}
}

export const sourceinfo = writable<Source[]>([]);

export async function loadsources() {
	sourceinfo.set(await listSources());
}

export async function createSource(
	owner: string,
	repo: string,
	branch?: string,
	aliases?: Record<string, string>
): Promise<Source | null> {
	try {
		const res = await fetch(`${apibase}/api/sources`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ owner, repo, branch, aliases }),
			credentials: 'include'
		});
		if (!res.ok) return null;
		return SourceSchema.parse(await res.json());
	} catch (e) {
		console.error('Create source failed:', e);
		return null;
	}
}

export async function refreshSource(sourceId: number): Promise<boolean> {
	try {
		const res = await fetch(`${apibase}/api/sources/${sourceId}/sync`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			credentials: 'include'
		});
		return res.ok;
	} catch (e) {
		console.error('Sync source failed:', e);
		return false;
	}
}

export async function deleteSource(sourceId: number): Promise<boolean> {
	try {
		const res = await fetch(`${apibase}/api/sources/${sourceId}`, {
			method: 'DELETE',
			headers: { 'Content-Type': 'application/json' },
			credentials: 'include'
		});
		return res.ok;
	} catch (e) {
		console.error('Delete source failed:', e);
		return false;
	}
}

export async function publish(): Promise<{ version: number; resources: number } | null> {
	try {
		const res = await fetch(`${apibase}/api/publish`, {
			method: 'POST',
			credentials: 'include'
		});
		if (!res.ok) return null;
		return await res.json();
	} catch (e) {
		console.error('Publish failed:', e);
		return null;
	}
}

/** re-runs the rules on rule-classified rows, returns how many changed */
export async function reclassify(): Promise<number | null> {
	try {
		const res = await fetch(`${apibase}/api/reclassify`, {
			method: 'POST',
			credentials: 'include'
		});
		if (!res.ok) return null;
		refreshpending();
		return (await res.json()).updated;
	} catch (e) {
		console.error('Reclassify failed:', e);
		return null;
	}
}

export const pending = writable(0);

/** counts changes since the last publish, admins only */
export async function refreshpending() {
	try {
		const res = await fetch(`${apibase}/api/publish/pending`, { credentials: 'include' });
		if (res.ok) pending.set((await res.json()).pending);
	} catch (e) {
		console.error('Fetch pending failed:', e);
	}
}

/** applies one edit to every id, locking the rows as manual */
export async function editresources(
	ids: string[],
	edit: Record<string, unknown>
): Promise<boolean> {
	try {
		const res = await fetch(`${apibase}/api/resources`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ ids: ids.map(Number), ...edit }),
			credentials: 'include'
		});
		if (res.ok) refreshpending();
		return res.ok;
	} catch (e) {
		console.error('Edit failed:', e);
		return false;
	}
}

/** creates a local offering, or returns the existing one */
export async function createoffering(
	course: string,
	season: string,
	year: number
): Promise<string | null> {
	try {
		const res = await fetch(`${apibase}/api/offerings`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ course, season, year }),
			credentials: 'include'
		});
		return res.ok ? String((await res.json()).id) : null;
	} catch (e) {
		console.error('Create offering failed:', e);
		return null;
	}
}
