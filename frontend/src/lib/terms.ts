export type Term = { season: 'M' | 'S'; year: number };

/** one exam window from the almanac, with the ug1 dates where they differ */
export type Exam = {
	exam: 'quiz' | 'mid' | 'end' | 'lab';
	n?: number;
	from: string;
	to: string;
	ug1?: { from: string; to: string };
};

export type Almanac = Record<string, Exam[]>;

/** the exam whose papers matter now, shown until its last day */
export type Phase = {
	exam: Exam['exam'];
	n: number | null;
	label: string;
	from: Date;
	until: Date;
	term: Term;
	guessed: boolean;
};

const examlabels: Record<string, string> = {
	quiz: 'Quiz',
	mid: 'Mid sem',
	end: 'End sem',
	lab: 'Lab exam'
};

/** the term a date falls in: dec 10 to may 15 is spring of the next year, may 16 to dec 9 monsoon */
export function termof(d: Date): Term {
	const [y, m, day] = [d.getFullYear(), d.getMonth(), d.getDate()];
	if (m === 11 && day >= 10) return { season: 'S', year: y + 1 };
	if (m < 4 || (m === 4 && day <= 15)) return { season: 'S', year: y };
	return { season: 'M', year: y };
}

/** short form like "M26" */
export const termkey = (t: Term) => `${t.season}${String(t.year).slice(2)}`;

/** long form like "Monsoon 2026" */
export const termname = (t: Term) => `${t.season === 'M' ? 'Monsoon' : 'Spring'} ${t.year}`;

/** sorts terms in the order they happen */
export const termorder = (t: Term) => t.year * 2 + (t.season === 'M' ? 1 : 0);

const nextterm = (t: Term): Term =>
	t.season === 'S' ? { season: 'M', year: t.year } : { season: 'S', year: t.year + 1 };

/** midnight local time of a yyyy-mm-dd date, moved by whole years */
function dayof(iso: string, years = 0): Date {
	const [y, m, d] = iso.split('-').map(Number);
	return new Date(y + years, m - 1, d);
}

/** a term's exams; a term the almanac lacks borrows the latest same season, a year on */
function examsfor(almanac: Almanac, t: Term): { exams: Exam[]; shift: number } {
	const own = almanac[termkey(t)];
	if (own) return { exams: own, shift: 0 };
	const year = (k: string) => 2000 + Number(k.slice(1));
	const latest = Object.keys(almanac)
		.filter((k) => k[0] === t.season && year(k) < t.year)
		.sort((a, b) => year(b) - year(a))[0];
	return latest
		? { exams: almanac[latest], shift: t.year - year(latest) }
		: { exams: [], shift: 0 };
}

/** windows of one term as dates, using the ug1 dates when asked */
function windows(almanac: Almanac, t: Term, ug1: boolean): Phase[] {
	const { exams, shift } = examsfor(almanac, t);
	return exams.map((e) => {
		const w = (ug1 && e.ug1) || e;
		return {
			exam: e.exam,
			n: e.n ?? null,
			label: `${examlabels[e.exam]}${e.n ? ` ${e.n}` : ''}`,
			from: dayof(w.from, shift),
			until: dayof(w.to, shift),
			term: t,
			guessed: shift !== 0
		};
	});
}

/**
 * the latest exam that has begun and not yet ended, else the next one;
 * looks into the next term once this one's are over
 */
export function phase(almanac: Almanac, now: Date, ug1: boolean): Phase | null {
	const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
	const t = termof(now);
	const list = [...windows(almanac, t, ug1), ...windows(almanac, nextterm(t), ug1)];
	const live = list.filter((w) => w.from <= today && today <= w.until);
	if (live.length) return live.reduce((a, b) => (b.from > a.from ? b : a));
	return list.filter((w) => w.from > today).sort((a, b) => +a.from - +b.from)[0] ?? null;
}

/** whether a paper belongs to an exam; unnumbered quizzes count for every quiz */
export function inphase(
	r: { kind: string | null; examtype: string | null; n: number | null },
	p: Phase
): boolean {
	return (
		r.kind === 'exams' &&
		r.examtype === p.exam &&
		(p.exam !== 'quiz' || r.n === null || r.n === p.n)
	);
}

/** ug1 ends on the first may 1 after it was turned on */
export function ug1until(since: Date): Date {
	return new Date(since.getFullYear() + (since.getMonth() >= 4 ? 1 : 0), 4, 1);
}

/** "Oct 31" */
export const shortdate = (d: Date) =>
	d.toLocaleDateString('en-IN', { month: 'short', day: 'numeric' });
