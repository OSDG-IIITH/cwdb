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
	course_name: z.string().nullable().optional()
});

export type Resource = z.infer<typeof ResourceSchema>;

export const SearchResponseSchema = z.object({
	resources: z.array(ResourceSchema)
});

export const CourseSchema = z.object({
	id: z.string(),
	name: z.string(),
	aliases: z.array(z.string()),
	resource_count: z.number(),
	pinned: z.boolean()
});

export type Course = z.infer<typeof CourseSchema>;

export const CoursesResponseSchema = z.object({
	courses: z.array(CourseSchema),
	pinned: z.array(CourseSchema),
	total: z.number(),
	page: z.number(),
	per_page: z.number()
});

export type CoursesResponse = z.infer<typeof CoursesResponseSchema>;

export const ToggleCoursePinResponseSchema = z.object({
	pinned: z.boolean()
});

export async function listcourses(opts?: {
	page?: number;
	per_page?: number;
	q?: string;
}): Promise<CoursesResponse> {
	const empty = { courses: [], pinned: [], total: 0, page: 1, per_page: 30 };
	try {
		const params = new URLSearchParams();
		if (opts?.page) params.set('page', String(opts.page));
		if (opts?.per_page) params.set('per_page', String(opts.per_page));
		if (opts?.q) params.set('q', opts.q);
		const qs = params.toString();
		const res = await fetch(`${API_BASE}/api/courses${qs ? `?${qs}` : ''}`, {
			credentials: 'include'
		});
		if (!res.ok) return empty;
		const json = await res.json();
		return CoursesResponseSchema.parse({
			courses: json.courses ?? [],
			pinned: json.pinned ?? [],
			total: json.total ?? json.courses?.length ?? 0,
			page: json.page ?? 1,
			per_page: json.per_page ?? 30
		});
	} catch (e) {
		console.error('Fetch courses failed:', e);
		return empty;
	}
}

export async function togglecoursepin(courseid: string): Promise<{ pinned: boolean } | null> {
	try {
		const res = await fetch(`${API_BASE}/api/courses/${courseid}/pin`, {
			method: 'POST',
			credentials: 'include'
		});
		if (!res.ok) return null;
		const json = await res.json();
		return ToggleCoursePinResponseSchema.parse(json);
	} catch (e) {
		console.error('Toggle course pin failed:', e);
		return null;
	}
}

export const CourseDetailSchema = z.object({
	id: z.string(),
	name: z.string(),
	aliases: z.array(z.string())
});

export type CourseDetail = z.infer<typeof CourseDetailSchema>;

export async function getcourse(
	id: string
): Promise<{ course: CourseDetail; resources: Resource[] } | null> {
	try {
		const res = await fetch(`${API_BASE}/api/courses/${id}`, { credentials: 'include' });
		if (!res.ok) return null;
		const json = await res.json();
		return {
			course: CourseDetailSchema.parse(json.course),
			resources: z.array(ResourceSchema).parse(json.resources)
		};
	} catch (e) {
		console.error('Fetch course failed:', e);
		return null;
	}
}

export async function searchResources(query: string, courseid?: string): Promise<Resource[]> {
	try {
		let url = `${API_BASE}/api/search?q=${encodeURIComponent(query)}`;
		if (courseid) url += `&course=${encodeURIComponent(courseid)}`;
		const res = await fetch(url, {
			credentials: 'include'
		});
		if (!res.ok) {
			return [];
		}
		const json = await res.json();
		const data = SearchResponseSchema.parse(json);
		return data.resources;
	} catch (e) {
		console.error('Search failed:', e);
		return [];
	}
}

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
	liked: z.boolean()
});

export type Source = z.infer<typeof SourceSchema>;

export const SourcesResponseSchema = z.object({
	sources: z.array(SourceSchema)
});

export async function listSources(filter?: string): Promise<Source[]> {
	try {
		const query = filter ? `?filter=${filter}` : '';
		const res = await fetch(`${API_BASE}/api/sources${query}`, {
			credentials: 'include'
		});
		if (!res.ok) {
			const errorText = await res.text();
			console.error('Fetch sources failed:', res.status, errorText);
			return [];
		}
		const json = await res.json();
		const data = SourcesResponseSchema.parse({
			sources: json.sources ?? []
		});
		return data.sources;
	} catch (e) {
		console.error('Fetch sources failed:', e);
		return [];
	}
}

export async function createSource(
	owner: string,
	repo: string,
	branch?: string
): Promise<Source | null> {
	try {
		const res = await fetch(`${API_BASE}/api/sources`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ owner, repo, branch }),
			credentials: 'include'
		});
		if (!res.ok) {
			const errorText = await res.text();
			console.error('Create source failed:', res.status, errorText);
			return null;
		}
		const json = await res.json();
		return SourceSchema.parse(json);
	} catch (e) {
		console.error('Create source failed:', e);
		return null;
	}
}

export async function refreshSource(sourceId: number): Promise<boolean> {
	try {
		const res = await fetch(`${API_BASE}/api/sources/${sourceId}/sync`, {
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

export async function toggleSourceLike(
	sourceId: number
): Promise<{ liked: boolean; like_count: number } | null> {
	try {
		const res = await fetch(`${API_BASE}/api/sources/${sourceId}/like`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			credentials: 'include'
		});
		if (!res.ok) {
			console.error('Toggle source like failed:', res.status);
			return null;
		}
		return await res.json();
	} catch (e) {
		console.error('Toggle source like failed:', e);
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
			credentials: 'include'
		});
		if (!res.ok) {
			console.error('Fetch resources failed:', res.status);
			return [];
		}
		const data = await res.json();
		return data.resources || [];
	} catch (e) {
		console.error('Fetch resources failed:', e);
		return [];
	}
}
export async function deleteSource(sourceId: number): Promise<boolean> {
	try {
		const res = await fetch(`${API_BASE}/api/sources/${sourceId}`, {
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
