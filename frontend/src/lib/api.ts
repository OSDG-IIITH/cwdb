import { z } from 'zod';
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
	like_count: z.number(),
	aliases: z.record(z.string(), z.string()).optional().default({}),
	liked: z.boolean()
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
