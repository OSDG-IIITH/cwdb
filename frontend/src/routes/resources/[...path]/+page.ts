import { redirect } from '@sveltejs/kit';
import { base } from '$app/paths';

export function load({ params }) {
	redirect(307, `${base}/sources${params.path ? `/${params.path}` : ''}`);
}
