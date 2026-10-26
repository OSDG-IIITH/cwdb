import { derived, get, writable } from 'svelte/store';
import { toast } from 'svelte-sonner';
import { apibase } from '$lib/config';
import { saved, user } from '$lib/auth';
import { mycourses, togglepin } from '$lib/sync';
import { termof, termorder } from '$lib/terms';
import { restoretheme } from '$lib/theme';

export type MyCourse = { course: string; added: string; archived: boolean };

export type Settings = {
	autoterm: boolean;
	examfocus: boolean;
	ug1: boolean;
	ug1since: string | null;
	prefs: Record<string, string>;
};

export type Profile = Settings & {
	courses: MyCourse[];
	progress: { id: string; done: boolean }[];
};

export const profile = writable<Profile | null>(null);

/** set when the profile could not be loaded and there is no copy on this device */
export const profilefailed = writable(false);

/** done state per marked resource id */
export const progress = derived(
	profile,
	(p) => new Map(p?.progress.map((x) => [x.id, x.done]) ?? [])
);

/** whether a course sits in the archive: by the term it was added in with autoterm on, by hand otherwise */
export function archived(c: MyCourse, autoterm: boolean, now = new Date()): boolean {
	return autoterm ? termorder(termof(new Date(c.added))) < termorder(termof(now)) : c.archived;
}

/** codes in my courses: the account's active ones when logged in, else the local pins */
export const mine = derived([user, profile, mycourses], ([$user, $profile, $pins]) =>
	$user
		? ($profile?.courses.filter((c) => !archived(c, $profile.autoterm)).map((c) => c.course) ?? [])
		: $pins
);

/** local storage keys that follow the account */
const synced = (k: string) =>
	['theme-variant', 'mode-watcher-mode', 'courseview'].includes(k) || k.startsWith('cwdb:width:');

/** prefs as a string that ignores key order */
const stable = (prefs: Record<string, string>) =>
	JSON.stringify(Object.fromEntries(Object.entries(prefs).sort(([a], [b]) => a.localeCompare(b))));

let pushed = '';

/** the response, null when the backend refused, undefined when it could not be reached */
async function call(path: string, method: string, body?: unknown, keepalive = false) {
	try {
		const res = await fetch(`${apibase}/api/profile${path}`, {
			method,
			headers: body ? { 'Content-Type': 'application/json' } : undefined,
			body: body ? JSON.stringify(body) : undefined,
			credentials: 'include',
			keepalive
		});
		return res.ok ? res : null;
	} catch (e) {
		console.error(`${method} profile${path} failed:`, e);
		return undefined;
	}
}

/** keeps the profile as the backend last confirmed it, for offline use */
function remember() {
	try {
		const p = get(profile);
		if (p) localStorage.setItem(saved.profile, JSON.stringify({ id: get(user)?.id, p }));
	} catch {}
}

function lastprofile(): Profile | null {
	try {
		const last = JSON.parse(localStorage.getItem(saved.profile) ?? 'null');
		return last && last.id === get(user)?.id ? last.p : null;
	} catch {
		return null;
	}
}

function patch(fn: (p: Profile) => Profile) {
	profile.update((p) => (p ? fn(p) : p));
}

/** shows the error and reloads, undoing the optimistic change */
async function failed(message: string) {
	toast.error(navigator.onLine ? message : 'You are offline, try again once connected');
	await loadprofile();
}

export async function loadprofile() {
	profilefailed.set(false);
	const res = await call('', 'GET');
	if (!res) {
		if (!get(profile)) profile.set(lastprofile());
		return profilefailed.set(!get(profile));
	}
	const p: Profile = await res.json();
	profile.set(p);
	remember();
	if (Object.keys(p.prefs).length) pullprefs(p.prefs);
	else await pushprefs();
	await importpins(p);
}

/** moves local pins into an empty account once per browser */
async function importpins(p: Profile) {
	if (localStorage.getItem('pins-imported')) return;
	localStorage.setItem('pins-imported', '1');
	const pins = get(mycourses);
	if (!p.courses.length && pins.length) await addcourses(pins);
}

/** writes synced prefs into this browser and applies the theme */
function pullprefs(prefs: Record<string, string>) {
	for (const [k, v] of Object.entries(prefs)) if (synced(k)) localStorage.setItem(k, v);
	pushed = stable(prefs);
	restoretheme();
}

/** sends this browser's prefs to the account when they changed */
export async function pushprefs(keepalive = false) {
	if (!get(profile)) return;
	const prefs: Record<string, string> = {};
	for (let i = 0; i < localStorage.length; i++) {
		const k = localStorage.key(i)!;
		if (synced(k)) prefs[k] = localStorage.getItem(k)!;
	}
	if (stable(prefs) === pushed) return;
	pushed = stable(prefs);
	if (await call('', 'PATCH', { prefs }, keepalive)) {
		patch((p) => ({ ...p, prefs }));
		remember();
	}
}

/** shows the device copy of the profile at once and refreshes it with each new user, and pushes prefs whenever the tab is left */
export function initprofile() {
	let current: string | undefined;
	user.subscribe((u) => {
		if (u?.id === current) return;
		current = u?.id;
		profile.set(u ? lastprofile() : null);
		if (u) loadprofile();
	});
	document.addEventListener('visibilitychange', () => {
		if (document.visibilityState === 'hidden') pushprefs(true);
	});
}

export async function setsettings(s: Partial<Omit<Settings, 'ug1since' | 'prefs'>>) {
	const now = new Date().toISOString();
	patch((p) => ({
		...p,
		...s,
		ug1since: s.ug1 === undefined ? p.ug1since : s.ug1 ? (p.ug1 ? p.ug1since : now) : null
	}));
	const res = await call('', 'PATCH', s);
	if (!res) return failed('could not save the setting');
	const settings: Settings = await res.json();
	patch((p) => ({ ...p, ...settings }));
	remember();
}

/** turning autoterm off keeps what the terms had archived as the manual state */
export async function setautoterm(on: boolean) {
	const p = get(profile);
	if (!p) return;
	if (!on && p.autoterm) {
		const old = p.courses.filter((c) => archived(c, true)).map((c) => c.course);
		const live = p.courses.filter((c) => !archived(c, true)).map((c) => c.course);
		await Promise.all([
			old.length && archivecourses(old, true),
			live.length && archivecourses(live, false)
		]);
	}
	await setsettings({ autoterm: on });
}

/** adds courses, or brings archived ones back as added now */
export async function addcourses(codes: string[]) {
	const added = new Date().toISOString();
	patch((p) => ({
		...p,
		courses: [
			...p.courses.filter((c) => !codes.includes(c.course)),
			...codes.map((course) => ({ course, added, archived: false }))
		]
	}));
	if (await call('/courses', 'POST', { courses: codes })) remember();
	else await failed('could not add the course');
}

export async function removecourse(code: string) {
	patch((p) => ({ ...p, courses: p.courses.filter((c) => c.course !== code) }));
	if (await call(`/courses/${encodeURIComponent(code)}`, 'DELETE')) remember();
	else await failed('could not remove the course');
}

export async function archivecourses(codes: string[], archived: boolean) {
	patch((p) => ({
		...p,
		courses: p.courses.map((c) => (codes.includes(c.course) ? { ...c, archived } : c))
	}));
	if (await call('/courses', 'PATCH', { courses: codes, archived })) remember();
	else await failed('could not update the course');
}

/** the plus button: adds to or removes from my courses, the local pins when logged out */
export function togglemine(code: string) {
	if (!get(user)) return togglepin(code);
	if (get(mine).includes(code)) removecourse(code);
	else addcourses([code]);
}

export async function setdone(id: string, done: boolean) {
	patch((p) => ({ ...p, progress: [...p.progress.filter((x) => x.id !== id), { id, done }] }));
	if (await call(`/progress/${id}`, 'PUT', { done })) remember();
	else await failed('could not save progress');
}
