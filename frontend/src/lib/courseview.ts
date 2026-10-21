import { kinds, type Offering, type Resource } from '$lib/sync';

export type Row = { r: Resource; kids: Resource[]; alts: Resource[] };
export type Group = { name: string; note: string; rows: Row[] };
export type TreeRow = {
	id: string;
	name: string;
	depth: number;
	dir: boolean;
	count: number;
	open: boolean;
	course?: string | null;
	r?: Resource;
};

const examorder = ['quiz', 'mid', 'end', 'lab'];
const examlabels: Record<string, string> = {
	quiz: 'Quizzes',
	mid: 'Mid',
	end: 'End',
	lab: 'Lab'
};

/** hides files inside unit folders and maps each unit's primary file to them */
function splitunits(files: Resource[]) {
	const units = new Map<string, Resource>();
	for (const r of files) if (r.unit) units.set(`${r.source_id}:${r.unit}`, r);
	const kids = new Map<string, Resource[]>();
	const top: Resource[] = [];
	for (const r of files) {
		const dirs = r.file_path.split('/').slice(0, -1);
		let owner: Resource | undefined;
		for (let i = dirs.length; i > 0 && !owner; i--) {
			owner = units.get(`${r.source_id}:${dirs.slice(0, i).join('/')}`);
		}
		if (owner && owner.id !== r.id) kids.set(owner.id, [...(kids.get(owner.id) ?? []), r]);
		else top.push(r);
	}
	return { top, kids };
}

/** same paper from different sources shares this key */
const paperkey = (r: Resource) => `${r.offering}|${r.kind}|${r.key}|${r.title}`;

/** counts rows per nav key: all, kind and exam type */
export function navcounts(files: Resource[]): Record<string, number> {
	const out: Record<string, number> = { all: 0 };
	const seen = new Set<string>();
	for (const r of splitunits(files).top) {
		if (r.kind && seen.has(paperkey(r))) continue;
		seen.add(paperkey(r));
		const keys = ['all', r.kind ?? 'other'];
		if (r.examtype) keys.push(`exams:${r.examtype}`);
		for (const k of keys) out[k] = (out[k] ?? 0) + 1;
	}
	return out;
}

/** rows for a kind view grouped by offering, then undated, then earlier courses */
export function buildgroups(
	own: Resource[],
	earlier: [string, Resource[]][],
	nav: string,
	query: string,
	offs: Map<string, Offering>,
	by = 'sem'
): Group[] {
	const [kind, sub] = nav.split(':');
	const q = query.trim().toLowerCase();
	const year = (r: Resource) => offs.get(r.offering ?? '')?.year ?? 0;
	const rank = (r: Resource) => [
		kinds.indexOf(r.kind ?? 'other'),
		examorder.indexOf(r.examtype ?? ''),
		r.n ?? 0,
		r.key ? 1 : 0
	];
	const cmp = (a: Resource, b: Resource) => {
		const [x, y] = [rank(a), rank(b)];
		const i = x.findIndex((v, j) => v !== y[j]);
		return year(b) - year(a) || (i < 0 ? a.title.localeCompare(b.title) : x[i] - y[i]);
	};
	const rows = (list: Resource[]): Row[] => {
		const { top, kids } = splitunits(list);
		const found = new Map<string, Row>();
		for (const r of top
			.filter(
				(r) =>
					(kind === 'all' || (r.kind ?? 'other') === kind) &&
					(!sub || r.examtype === sub) &&
					(!q || `${r.title} ${r.file_path}`.toLowerCase().includes(q))
			)
			.sort(cmp)) {
			const dup = r.kind ? found.get(paperkey(r)) : undefined;
			if (dup) dup.alts.push(r);
			else found.set(r.kind ? paperkey(r) : r.id, { r, kids: kids.get(r.id) ?? [], alts: [] });
		}
		return [...found.values()];
	};

	if (by === 'kind' && kind === 'all') {
		const found = new Map<string, Group & { rank: number[] }>();
		for (const row of rows([...own, ...earlier.flatMap(([, list]) => list)])) {
			const k = row.r.kind ?? 'other';
			const sub = k === 'exams' ? row.r.examtype : undefined;
			const name = `${k.charAt(0).toUpperCase()}${k.slice(1)}${sub ? ` · ${examlabels[sub] ?? sub}` : ''}`;
			const g = found.get(name) ?? {
				name,
				note: '',
				rows: [],
				rank: [kinds.indexOf(k), examorder.indexOf(sub ?? '')]
			};
			g.rows.push(row);
			found.set(name, g);
		}
		return [...found.values()].sort((a, b) => a.rank[0] - b.rank[0] || a.rank[1] - b.rank[1]);
	}

	const byoffering = new Map<string, Row[]>();
	for (const row of rows(own)) {
		const key = offs.has(row.r.offering ?? '') ? row.r.offering! : '';
		byoffering.set(key, [...(byoffering.get(key) ?? []), row]);
	}
	const groups = [...byoffering]
		.map(([id, list]) => {
			const o = offs.get(id);
			return {
				name: o ? `${o.season === 'M' ? 'Monsoon' : 'Spring'} ${o.year}` : 'Undated',
				note: o?.faculty.join(', ') ?? '',
				rows: list,
				year: o?.year ?? 0
			};
		})
		.sort(
			(a, b) => b.year - a.year || (a.name === 'Undated' ? 1 : 0) - (b.name === 'Undated' ? 1 : 0)
		);
	for (const [code, list] of earlier) {
		const found = rows(list);
		if (found.length) groups.push({ name: `Earlier, as ${code}`, note: '', rows: found, year: 0 });
	}
	return groups;
}

/** a folder tree of files in one source, cropped to a root; folders are open unless toggled, or the reverse when openall is false */
export function buildtree(
	files: Resource[],
	root: string,
	toggled: Set<string>,
	openall = true
): TreeRow[] {
	type Node = {
		name: string;
		kids: Map<string, Node>;
		r?: Resource;
		count: number;
		codes: Set<string>;
		bare: boolean;
	};
	const top: Node = { name: '', kids: new Map(), count: 0, codes: new Set(), bare: false };
	for (const r of files) {
		const parts = r.file_path.slice(root ? root.length + 1 : 0).split('/');
		let node = top;
		for (const [i, p] of parts.entries()) {
			node.count++;
			if (r.course_id) node.codes.add(r.course_id);
			else node.bare = true;
			const next = node.kids.get(p) ?? {
				name: p,
				kids: new Map(),
				count: 0,
				codes: new Set(),
				bare: false
			};
			node.kids.set(p, next);
			node = next;
			if (i === parts.length - 1) node.r = r;
		}
	}
	const out: TreeRow[] = [];
	const walk = (node: Node, depth: number, path: string) => {
		const sorted = [...node.kids.values()].sort(
			(a, b) => Number(!!a.r) - Number(!!b.r) || a.name.localeCompare(b.name)
		);
		for (const k of sorted) {
			const id = `${path}/${k.name}`;
			if (k.r) {
				out.push({ id: k.r.id, name: k.name, depth, dir: false, count: 1, open: false, r: k.r });
				continue;
			}
			const open = openall !== toggled.has(id);
			const course =
				k.codes.size === 1 && !k.bare ? [...k.codes][0] : k.codes.size ? undefined : null;
			out.push({ id, name: k.name, depth, dir: true, count: k.count, open, course });
			if (open) walk(k, depth + 1, id);
		}
	};
	walk(top, 0, '');
	return out;
}

export type SourceRow = {
	id: string;
	owner: string;
	repo: string;
	branch: string;
	count: number;
	courses: number;
	mapped: number;
	kinded: number;
};

/** one row per source with its file, course and kind counts */
export function sourcelist(files: Resource[]): SourceRow[] {
	const found = new Map<string, SourceRow & { codes: Set<string> }>();
	for (const r of files) {
		const s = found.get(r.source_id) ?? {
			id: r.source_id,
			owner: r.owner,
			repo: r.repo,
			branch: r.branch,
			count: 0,
			courses: 0,
			mapped: 0,
			kinded: 0,
			codes: new Set<string>()
		};
		s.count++;
		if (r.course_id) {
			s.mapped++;
			s.codes.add(r.course_id);
		}
		if (r.kind) s.kinded++;
		found.set(r.source_id, s);
	}
	return [...found.values()]
		.map((s) => ({ ...s, courses: s.codes.size }))
		.sort((a, b) => b.count - a.count);
}
