import MiniSearch from 'minisearch';
import { openDB, type IDBPDatabase } from 'idb';
import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import { base } from '$app/paths';
import { githuburl } from '$lib/utils';

export type Resource = {
	id: string;
	source_id: string;
	owner: string;
	repo: string;
	branch: string;
	file_path: string;
	name: string;
	title: string;
	type: string | null;
	course_id: string | null;
	offering: string | null;
	kind: string | null;
	examtype: string | null;
	slidetype: string | null;
	n: number | null;
	key: boolean;
	keyfor: string | null;
	unit: string | null;
	size: number | null;
	review: boolean;
};

export type CourseInfo = {
	code: string;
	name: string;
	shortnames: string[];
	alias: string;
	semester: string;
	count: number;
	kinds: string[];
};

export type Offering = {
	id: string;
	course: string;
	season: string;
	year: number;
	label: string;
	faculty: string[];
};

export type Entry = {
	id: string;
	t: 'c' | 'd' | 'f';
	title: string;
	path: string;
	course: string | null;
	coursename: string;
	shortnames: string;
	terms: string;
	count: number;
	source_id?: string;
	dir?: string;
	github?: string;
	resource?: Resource;
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
	kind?: string;
	examtype?: string;
	slidetype?: string;
	n?: number;
	offering?: string;
	keyfor?: string;
	title?: string;
	unit?: string;
	size?: number;
	key?: boolean;
	review?: boolean;
};
type IndexSource = { name: string; url: string; owner: string; repo: string; branch: string };
type IndexCourse = { name: string; code: string; shortnames?: string[]; semester?: string };
type IndexOffering = { course: string; season: string; year: number };
type Catalog = {
	courses: Record<string, IndexCourse>;
	sources: Record<string, IndexSource>;
	offerings?: Record<string, IndexOffering>;
	faculty?: Record<string, string[]>;
	succession?: [string, string][];
};
type Index = Catalog & { version: number; resources: IndexResource[] };

export const kinds = [
	'exams',
	'assignments',
	'labs',
	'slides',
	'notes',
	'references',
	'projects',
	'other'
];

const examnames: Record<string, string> = {
	quiz: 'Quiz',
	mid: 'Mid',
	end: 'End sem',
	lab: 'Lab exam'
};
const slidenames: Record<string, string> = { lec: 'Lecture', tut: 'Tutorial' };
const unitnames: Record<string, string> = {
	assignments: 'Assignment',
	labs: 'Lab',
	projects: 'Project'
};
const kindtags: Record<string, string> = {
	exams: 'exam',
	assignments: 'assignment',
	labs: 'lab',
	slides: 'slides',
	notes: 'notes',
	references: 'reference',
	projects: 'project',
	other: 'other'
};
const termwords: Record<string, string> = {
	quiz: 'quiz',
	mid: 'midsem',
	end: 'endsem',
	lab: 'lab',
	lec: 'lecture',
	tut: 'tutorial'
};

const searchversion = 2;
const searchopts = {
	fields: ['title', 'path', 'coursename', 'shortnames', 'terms'],
	searchOptions: {
		fuzzy: (term: string) => (term.length > 4 && !/\d/.test(term) ? 0.2 : false),
		prefix: true,
		combineWith: 'AND',
		boost: { shortnames: 3, coursename: 2, title: 2 }
	}
};

export const syncstatus = writable<SyncStatus>({
	syncing: false,
	synced_at: null,
	version: null,
	online: false,
	empty: true
});

export const allresources = writable<Resource[]>([]);
export const courses = writable<CourseInfo[]>([]);
export const mycourses = writable<string[]>(browser ? loadpins() : []);

mycourses.subscribe((pins) => {
	if (browser) localStorage.setItem('pins', JSON.stringify(pins));
});

let ms: MiniSearch<Entry> | null = null;
let resourcemap = new Map<string, Resource>();
let entrymap = new Map<string, Entry>();
let coursemap = new Map<string, CourseInfo>();
let offeringmap = new Map<string, Offering>();
let db: IDBPDatabase | null = null;

const INDEX_URL = `${base}/data/index.json`;

function loadpins(): string[] {
	try {
		return JSON.parse(localStorage.getItem('pins') ?? '[]');
	} catch {
		return [];
	}
}

/** maps old uuid pins to course codes using the v1 course table */
function migratepins(json?: string) {
	if (!json) return;
	const old: Record<string, { code?: string }> = JSON.parse(json);
	mycourses.update((pins) => [...new Set(pins.map((p) => old[p]?.code ?? p).filter(Boolean))]);
}

/** toggles a course in my courses */
export function togglepin(code: string) {
	mycourses.update((pins) =>
		pins.includes(code) ? pins.filter((p) => p !== code) : [...pins, code]
	);
}

async function opendb() {
	db = await openDB('cwdb', 2, {
		upgrade(database, oldversion, newversion, tx) {
			if (oldversion === 0) {
				const s = database.createObjectStore('resources', { keyPath: 'id' });
				s.createIndex('course_id', 'course_id');
				s.createIndex('source_id', 'source_id');
				database.createObjectStore('meta');
			} else if (oldversion === 1) {
				const meta = tx.objectStore('meta');
				meta.get('courses').then(migratepins);
				meta.clear();
				tx.objectStore('resources').clear();
			}
		}
	});
}

function offeringlabel(o: IndexOffering): string {
	return `${o.season}${String(o.year).slice(2)}`;
}

/** builds a display title like "Quiz 1 · M24", else the file name without extension */
function displaytitle(r: IndexResource, name: string, offerings: Map<string, Offering>): string {
	if (r.title) return r.title;
	let label = '';
	if (r.kind === 'exams' && r.examtype) label = examnames[r.examtype] + (r.n ? ` ${r.n}` : '');
	else if (r.kind === 'slides' && r.slidetype && r.n) label = `${slidenames[r.slidetype]} ${r.n}`;
	else if (r.kind && unitnames[r.kind] && r.n) label = `${unitnames[r.kind]} ${r.n}`;
	if (!label) return name.replace(/\.[^.]+$/, '');
	if (r.key) label += ' key';
	const o = r.offering ? offerings.get(r.offering) : null;
	return o ? `${label} · ${o.label}` : label;
}

/** short tag for a file, like "exam · mid" or "slides · lec · key" */
export function kindtag(r: Resource): string {
	return [kindtags[r.kind ?? 'other'], r.examtype ?? r.slidetype, r.key ? 'key' : null]
		.filter(Boolean)
		.join(' · ');
}

function denormalize(
	raw: IndexResource[],
	catalog: Catalog,
	offerings: Map<string, Offering>
): Resource[] {
	return raw.map((r) => {
		const src = catalog.sources[r.source_id];
		const name = r.path.split('/').at(-1) ?? r.path;
		return {
			id: r.id,
			source_id: r.source_id,
			owner: src?.owner ?? '',
			repo: src?.repo ?? '',
			branch: src?.branch ?? 'main',
			file_path: r.path,
			name,
			title: displaytitle(r, name, offerings),
			type: r.type,
			course_id: r.course_id,
			offering: r.offering ?? null,
			kind: r.kind ?? null,
			examtype: r.examtype ?? null,
			slidetype: r.slidetype ?? null,
			n: r.n ?? null,
			key: r.key ?? false,
			keyfor: r.keyfor ?? null,
			unit: r.unit ?? null,
			size: r.size ?? null,
			review: r.review ?? false
		};
	});
}

function makeofferings(catalog: Catalog): Map<string, Offering> {
	return new Map(
		Object.entries(catalog.offerings ?? {}).map(([id, o]) => [
			id,
			{ id, ...o, label: offeringlabel(o), faculty: catalog.faculty?.[id] ?? [] }
		])
	);
}

function makecourses(catalog: Catalog, items: Resource[]): Map<string, CourseInfo> {
	const counts = new Map<string, number>();
	const present = new Map<string, Set<string>>();
	for (const r of items) {
		if (!r.course_id) continue;
		counts.set(r.course_id, (counts.get(r.course_id) ?? 0) + 1);
		if (!present.has(r.course_id)) present.set(r.course_id, new Set());
		present.get(r.course_id)!.add(r.kind ?? 'other');
	}
	return new Map(
		Object.values(catalog.courses).map((c) => {
			const shortnames = c.shortnames ?? [];
			const has = present.get(c.code) ?? new Set();
			return [
				c.code,
				{
					code: c.code,
					name: c.name,
					shortnames,
					alias: (shortnames[0] ?? c.code).toUpperCase(),
					semester: c.semester ?? '',
					count: counts.get(c.code) ?? 0,
					kinds: kinds.filter((k) => has.has(k))
				}
			];
		})
	);
}

function courseterms(code: string | null) {
	const c = code ? coursemap.get(code) : null;
	return { coursename: c?.name ?? '', shortnames: c ? [c.code, ...c.shortnames].join(' ') : '' };
}

/** search entries for every folder, tagged with a course when all its classified files share one */
export function folderentries(items: Resource[]): Entry[] {
	const folders = new Map<
		string,
		{ r: Resource; dir: string; count: number; course: string | null | undefined }
	>();
	for (const r of items) {
		const parts = r.file_path.split('/').slice(0, -1);
		for (let i = 1; i <= parts.length; i++) {
			const dir = parts.slice(0, i).join('/');
			const id = `d:${r.source_id}:${dir}`;
			const f = folders.get(id) ?? { r, dir, count: 0, course: undefined };
			f.count++;
			if (r.course_id && f.course === undefined) f.course = r.course_id;
			else if (r.course_id && f.course !== r.course_id) f.course = null;
			folders.set(id, f);
		}
	}
	return [...folders].map(([id, f]) => ({
		id,
		t: 'd',
		title: f.dir.split('/').at(-1) + '/',
		path: `${f.r.owner}/${f.r.repo}/${f.dir}`,
		course: f.course ?? null,
		...courseterms(f.course ?? null),
		terms: '',
		count: f.count,
		source_id: f.r.source_id,
		dir: f.dir,
		github: `https://github.com/${f.r.owner}/${f.r.repo}/tree/${f.r.branch}/${encodeURI(f.dir)}`
	}));
}

function fileterms(r: Resource): string {
	const o = r.offering ? offeringmap.get(r.offering) : null;
	return [
		r.kind,
		termwords[r.examtype ?? r.slidetype ?? ''],
		r.key ? 'key' : '',
		o ? `${o.year} ${o.label} ${o.season === 'M' ? 'monsoon' : 'spring'}` : ''
	]
		.filter(Boolean)
		.join(' ');
}

function makeentries(items: Resource[]): Entry[] {
	const coursentries: Entry[] = [...coursemap.values()].map((c) => ({
		id: `c:${c.code}`,
		t: 'c',
		title: c.name,
		path: c.code,
		course: c.code,
		...courseterms(c.code),
		terms: '',
		count: c.count
	}));
	const files: Entry[] = items
		.filter((r) => !(r.kind === 'other' && /\d{10}/.test(r.name)))
		.map((r) => ({
			id: r.id,
			t: 'f',
			title: r.title,
			path: `${r.owner}/${r.repo}/${r.file_path}`,
			course: r.course_id,
			...courseterms(r.course_id),
			terms: fileterms(r),
			count: 1,
			github: githuburl(r),
			resource: r
		}));
	return [...coursentries, ...folderentries(items), ...files];
}

/** loads items and catalog into memory and returns the search index */
function apply(items: Resource[], catalog: Catalog, serialized?: string): MiniSearch<Entry> {
	resourcemap = new Map(items.map((r) => [r.id, r]));
	offeringmap = makeofferings(catalog);
	coursemap = makecourses(catalog, items);
	const entries = makeentries(items);
	entrymap = new Map(entries.map((e) => [e.id, e]));
	if (serialized) {
		ms = MiniSearch.loadJSON<Entry>(serialized, searchopts as never);
	} else {
		ms = new MiniSearch<Entry>(searchopts as never);
		ms.addAll(entries);
	}
	courses.set([...coursemap.values()].sort((a, b) => a.name.localeCompare(b.name)));
	allresources.set(items);
	return ms;
}

async function loadsaved(): Promise<boolean> {
	if (!db) return false;
	const version = await db.get('meta', 'version');
	if (!version) return false;

	const [synced_at, items, catalog, stored, serialized] = await Promise.all([
		db.get('meta', 'synced_at') as Promise<Date | undefined>,
		db.getAll('resources') as Promise<Resource[]>,
		db.get('meta', 'catalog') as Promise<string | undefined>,
		db.get('meta', 'searchversion') as Promise<number | undefined>,
		db.get('meta', 'minisearch') as Promise<string | undefined>
	]);
	if (!catalog) return false;

	apply(items, JSON.parse(catalog), stored === searchversion ? serialized : undefined);
	syncstatus.update((s) => ({
		...s,
		version,
		synced_at: synced_at ?? null,
		empty: items.length === 0
	}));
	return true;
}

async function storeindex(index: Index) {
	const { resources, version, ...catalog } = index;
	const items = denormalize(resources, catalog, makeofferings(catalog));
	const search = apply(items, catalog);
	const synced_at = new Date();
	syncstatus.update((s) => ({ ...s, version, synced_at, empty: items.length === 0 }));

	if (!db) return;
	try {
		const tx = db.transaction(['resources', 'meta'], 'readwrite');
		await tx.objectStore('resources').clear();
		await Promise.all(items.map((r) => tx.objectStore('resources').put(r)));
		const meta = tx.objectStore('meta');
		await meta.put(version, 'version');
		await meta.put(synced_at, 'synced_at');
		await meta.put(JSON.stringify(search.toJSON()), 'minisearch');
		await meta.put(searchversion, 'searchversion');
		await meta.put(JSON.stringify(catalog), 'catalog');
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
		if (stored !== index.version || resourcemap.size === 0) {
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

export function getentry(id: string): Entry | null {
	return entrymap.get(id) ?? null;
}

/** searches courses, folders and files, optionally within one course */
export function search(query: string, course?: string | null): Entry[] {
	if (!ms || !query.trim()) return [];
	const hits = ms
		.search(query)
		.map((r) => entrymap.get(String(r.id)))
		.filter(Boolean) as Entry[];
	return course ? hits.filter((e) => e.course === course) : hits;
}

export function getresourcesbycourse(code: string): Resource[] {
	return [...resourcemap.values()].filter((r) => r.course_id === code);
}

/** finds a course by its url slug, the lowercased code */
export function getcourse(list: CourseInfo[], slug?: string): CourseInfo | null {
	const code = slug?.toLowerCase();
	return list.find((c) => c.code.toLowerCase() === code) ?? null;
}

export function getresource(id: string): Resource | null {
	return resourcemap.get(id) ?? null;
}

/** longest folder path shared by all the given file paths */
export function courseroot(paths: string[]): string {
	if (!paths.length) return '';
	let parts = paths[0].split('/').slice(0, -1);
	for (const p of paths) {
		const q = p.split('/');
		let i = 0;
		while (i < parts.length && i < q.length - 1 && parts[i] === q[i]) i++;
		parts = parts.slice(0, i);
	}
	return parts.join('/');
}

/** each source holding a course's files, with the course's folder inside it */
export function coursefolders(code: string) {
	const groups = new Map<string, Resource[]>();
	for (const r of getresourcesbycourse(code)) {
		groups.set(r.source_id, [...(groups.get(r.source_id) ?? []), r]);
	}
	return [...groups.values()]
		.map((files) => ({
			source_id: files[0].source_id,
			owner: files[0].owner,
			repo: files[0].repo,
			path: courseroot(files.map((r) => r.file_path)),
			count: files.length
		}))
		.sort((a, b) => b.count - a.count);
}

/** folders and files directly under a path in a source */
export function sourcetree(sourceid: string, path = '') {
	const prefix = path ? path + '/' : '';
	const folders = new Map<string, number>();
	const files: Resource[] = [];
	for (const r of resourcemap.values()) {
		if (r.source_id !== sourceid || !r.file_path.startsWith(prefix)) continue;
		const rest = r.file_path.slice(prefix.length).split('/');
		if (rest.length === 1) files.push(r);
		else folders.set(rest[0], (folders.get(rest[0]) ?? 0) + 1);
	}
	return {
		folders: [...folders].map(([name, count]) => ({ name, path: prefix + name, count })),
		files
	};
}

/** a course's offerings, newest first */
export function offeringsfor(code: string): Offering[] {
	return [...offeringmap.values()]
		.filter((o) => o.course === code)
		.sort((a, b) => b.year - a.year || a.season.localeCompare(b.season));
}
