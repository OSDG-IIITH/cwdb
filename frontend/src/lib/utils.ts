import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { base } from '$app/paths';
import type { Resource } from '$lib/sync';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export function rawgithuburl(resource: Resource): string {
	const branch = resource.branch || 'main';
	return `https://raw.githubusercontent.com/${resource.owner}/${resource.repo}/${branch}/${encodeURI(resource.file_path)}`;
}

export function githuburl(resource: Resource): string {
	return `https://github.com/${resource.owner}/${resource.repo}/blob/${resource.branch || 'main'}/${encodeURI(resource.file_path)}`;
}

export function coursehref(code: string): string {
	return `${base}/courses/${encodeURIComponent(code.toLowerCase())}`;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, 'child'> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, 'children'> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
