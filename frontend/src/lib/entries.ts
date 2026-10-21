import { base } from '$app/paths';
import { kindtag, type Entry } from '$lib/sync';
import { coursehref } from '$lib/utils';

/** second line under a search result */
export function entrysub(e: Entry): string {
	if (e.t === 'c') return `${e.count} files`;
	if (e.t === 'd') return e.path;
	return e.path.split('/').slice(0, -1).join('/');
}

/** short tag on the right of a search result */
export function entrytag(e: Entry): string {
	if (e.t === 'c') return 'course';
	if (e.t === 'd') return `${e.count} files`;
	return kindtag(e.resource!);
}

/** where a search result opens */
export function entryhref(e: Entry): string {
	if (e.t === 'c') return coursehref(e.course!);
	if (e.t === 'd')
		return e.course
			? `${coursehref(e.course)}/${encodeURI(e.path)}`
			: `${base}/sources/${encodeURI(e.path)}`;
	return `${e.course ? coursehref(e.course) : `${base}/courses/unclassified`}?f=${e.id}`;
}
