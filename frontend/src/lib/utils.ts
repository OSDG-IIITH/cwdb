import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';
import type { Resource } from '$lib/sync';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export function titlecase(value: string): string {
	return value.replace(/\b\w/g, (c) => c.toUpperCase());
}

export function toslug(name: string): string {
	return name.toLowerCase().replaceAll(' ', '-');
}

export function rawgithuburl(resource: Resource): string {
	const branch = resource.branch || 'main';
	return `https://raw.githubusercontent.com/${resource.owner}/${resource.repo}/${branch}/${encodeURI(resource.file_path)}`;
}

function ispdf(resource: Resource): boolean {
	return resource.file_path.toLowerCase().endsWith('.pdf');
}

/**
 * Opens a resource in a new tab. PDFs are fetched and reassembled as an
 * `application/pdf` blob so the browser renders them inline instead of
 * honouring GitHub's `Content-Disposition: attachment` header, which would
 * otherwise force a download.
 */
export function openresource(resource: Resource): void {
	const url = rawgithuburl(resource);

	if (!ispdf(resource)) {
		window.open(url, '_blank');
		return;
	}

	const win = window.open('', '_blank');
	fetch(url)
		.then((res) => {
			if (!res.ok) throw new Error(`failed to fetch ${resource.file_path}`);
			return res.blob();
		})
		.then((blob) => {
			const pdfblob = new Blob([blob], { type: 'application/pdf' });
			const bloburl = URL.createObjectURL(pdfblob);
			if (win) {
				win.location.href = bloburl;
			}
			setTimeout(() => URL.revokeObjectURL(bloburl), 60_000);
		})
		.catch(() => {
			if (win) win.location.href = url;
		});
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, 'child'> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, 'children'> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
