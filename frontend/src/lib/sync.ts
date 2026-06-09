import MiniSearch from 'minisearch';
import { openDB, type IDBPDatabase } from 'idb';
import { writable, get } from 'svelte/store';

export type Resource = {
	id: string;
	source_id: string;
	owner: string;
	repo: string;
	branch: string;
	file_path: string;
	title: string;
	type: string | null;
	like_count: number;
	course_name: string | null;
	course_id: string | null;
};

export type CourseInfo = {
	id: string;
	name: string;
	code: string;
	aliases: string[];
	instructors: string[];
	semester: string;
	year: number | null;
	resource_count: number;
};

export type SyncStatus = {
	syncing: boolean;
	synced_at: Date | null;
	version: number | null;
	online: boolean;
	empty: boolean;
};

type IndexResource = {
	id: string;
	path: string;
	source_id: string;
	course_id: string | null;
	type: string | null;
};
type IndexSource = { name: string; url: string; owner: string; repo: string; branch: string };
type IndexCourse = {
	name: string;
	code: string;
	aliases?: string[];
	instructors?: string[];
	semester?: string;
	year?: number | null;
};
type Index = {
	version: number;
	resources: IndexResource[];
	courses: Record<string, IndexCourse>;
	sources: Record<string, IndexSource>;
};

export const syncstatus = writable<SyncStatus>({
	syncing: false,
	synced_at: null,
	version: null,
	online: false,
	empty: true
});

export const allresources = writable<Resource[]>([]);

let ms: MiniSearch<Resource> | null = null;
let resourcemap = new Map<string, Resource>();
let courseindex = new Map<string, IndexCourse>();
let db: IDBPDatabase | null = null;

import { base } from '$app/paths';
const INDEX_URL = `${base}/data/index.json`;

async function opendb() {
	db = await openDB('cwdb', 1, {
		upgrade(database) {
			if (!database.objectStoreNames.contains('resources')) {
				const s = database.createObjectStore('resources', { keyPath: 'id' });
				s.createIndex('course_id', 'course_id');
				s.createIndex('source_id', 'source_id');
			}
			if (!database.objectStoreNames.contains('meta')) {
				database.createObjectStore('meta');
			}
		}
	});
}

function buildms(items: Resource[]): MiniSearch<Resource> {
	const search = new MiniSearch<Resource>({
		fields: ['file_path', 'course_id', 'source_id'],
		idField: 'id',
		searchOptions: { fuzzy: 0.2, prefix: true, boost: { course_id: 2 } }
	});
	search.addAll(items);
	return search;
}

function denormalize(
	raw: IndexResource[],
	courses: Record<string, IndexCourse>,
	sources: Record<string, IndexSource>
): Resource[] {
	return raw.map((r) => {
		const src = sources[r.source_id];
		const course = r.course_id ? courses[r.course_id] : null;
		return {
			id: r.id,
			source_id: r.source_id,
			owner: src?.owner ?? '',
			repo: src?.repo ?? '',
			branch: src?.branch ?? 'main',
			file_path: r.path,
			title: r.path.split('/').at(-1) ?? r.path,
			type: r.type,
			like_count: 0,
			course_name: course?.name ?? null,
			course_id: r.course_id
		};
	});
}

async function loadsaved(): Promise<boolean> {
	if (!db) return false;
	const version = await db.get('meta', 'version');
	if (!version) return false;

	const [synced_at, items, coursesJson, serialized] = await Promise.all([
		db.get('meta', 'synced_at') as Promise<Date | undefined>,
		db.getAll('resources') as Promise<Resource[]>,
		db.get('meta', 'courses') as Promise<string | undefined>,
		db.get('meta', 'minisearch') as Promise<string | undefined>
	]);

	resourcemap = new Map(items.map((r) => [r.id, r]));
	if (coursesJson) courseindex = new Map(Object.entries(JSON.parse(coursesJson)));

	if (serialized) {
		ms = MiniSearch.loadJSON<Resource>(serialized, {
			fields: ['file_path', 'course_id', 'source_id'],
			searchOptions: { fuzzy: 0.2, prefix: true, boost: { course_id: 2 } }
		});
	} else {
		ms = buildms(items);
	}

	allresources.set(items);
	syncstatus.update((s) => ({
		...s,
		version,
		synced_at: synced_at ?? null,
		empty: items.length === 0
	}));
	return true;
}

async function storeindex(index: Index) {
	const items = denormalize(index.resources, index.courses, index.sources);
	const search = buildms(items);
	const synced_at = new Date();

	ms = search;
	resourcemap = new Map(items.map((r) => [r.id, r]));
	courseindex = new Map(Object.entries(index.courses));
	allresources.set(items);
	syncstatus.update((s) => ({ ...s, version: index.version, synced_at, empty: items.length === 0 }));

	if (!db) return;
	try {
		const tx = db.transaction(['resources', 'meta'], 'readwrite');
		await tx.objectStore('resources').clear();
		await Promise.all(items.map((r) => tx.objectStore('resources').put(r)));
		const meta = tx.objectStore('meta');
		await meta.put(index.version, 'version');
		await meta.put(synced_at, 'synced_at');
		await meta.put(JSON.stringify(search.toJSON()), 'minisearch');
		await meta.put(JSON.stringify(index.courses), 'courses');
		await tx.done;
	} catch (e) {
		console.error('idb write failed:', e);
	}
}

export async function trysync() {
	syncstatus.update((s) => ({ ...s, syncing: true }));
	try {
		const res = await fetch(INDEX_URL, { cache: 'no-cache' });
		if (!res.ok) throw new Error(`${res.status}`);
		const index: Index = await res.json();
		syncstatus.update((s) => ({ ...s, online: true }));
		const stored = db ? await db.get('meta', 'version') : null;
		if (stored !== index.version) {
			await storeindex(index);
		}
	} catch (e) {
		console.error('sync failed:', e);
		syncstatus.update((s) => ({ ...s, online: false }));
	} finally {
		syncstatus.update((s) => ({ ...s, syncing: false }));
	}
}

export async function init() {
	window.addEventListener('offline', () => syncstatus.update((s) => ({ ...s, online: false })));
	window.addEventListener('online', () => trysync());

	await opendb();
	await loadsaved();
	await trysync();
}

export function search(query: string, courseid?: string | null): Resource[] {
	if (!query.trim()) {
		const all = get(allresources);
		return courseid ? all.filter((r) => r.course_id === courseid) : [];
	}
	if (!ms) return [];
	const results = ms.search(query);
	let hits = results.map((r) => resourcemap.get(String(r.id))).filter(Boolean) as Resource[];
	if (courseid) hits = hits.filter((r) => r.course_id === courseid);
	return hits;
}

export function getresourcesbycourse(courseid: string): Resource[] {
	return Array.from(resourcemap.values()).filter((r) => r.course_id === courseid);
}

export function getunclassified(): Resource[] {
	return Array.from(resourcemap.values()).filter((r) => !r.course_id);
}

export function getcourses(): CourseInfo[] {
	const counts = new Map<string, number>();
	for (const r of resourcemap.values()) {
		if (r.course_id) counts.set(r.course_id, (counts.get(r.course_id) ?? 0) + 1);
	}
	return Array.from(courseindex.entries())
		.map(([id, c]) => ({
			id,
			name: c.name,
			code: c.code,
			aliases: c.aliases ?? [],
			instructors: c.instructors ?? [],
			semester: c.semester ?? '',
			year: c.year ?? null,
			resource_count: counts.get(id) ?? 0
		}))
		.sort((a, b) => a.name.localeCompare(b.name));
}

export function getcourse(slug: string): CourseInfo | null {
	const name = slug.replace(/-/g, ' ');
	const counts = new Map<string, number>();
	for (const r of resourcemap.values()) {
		if (r.course_id) counts.set(r.course_id, (counts.get(r.course_id) ?? 0) + 1);
	}
	for (const [id, c] of courseindex.entries()) {
		if (c.name.toLowerCase() === name.toLowerCase()) {
			return {
				id,
				name: c.name,
				code: c.code,
				aliases: c.aliases ?? [],
				instructors: c.instructors ?? [],
				semester: c.semester ?? '',
				year: c.year ?? null,
				resource_count: counts.get(id) ?? 0
			};
		}
	}
	return null;
}
