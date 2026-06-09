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

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, 'child'> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, 'children'> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
