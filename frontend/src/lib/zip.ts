import JSZip from 'jszip';
import { toast } from 'svelte-sonner';
import { rawgithuburl } from '$lib/utils';
import type { Resource } from '$lib/sync';

/** saves a blob under a file name */
function saveblob(blob: Blob, name: string) {
	const url = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = url;
	a.download = name;
	a.click();
	URL.revokeObjectURL(url);
}

/** downloads one file */
export async function downloadfile(r: Resource) {
	try {
		const res = await fetch(rawgithuburl(r));
		if (!res.ok) throw new Error(`${res.status}`);
		saveblob(await res.blob(), r.name);
	} catch {
		toast.error('download failed');
	}
}

/** zips the files and saves the archive */
export async function downloadzip(files: Resource[], name: string) {
	try {
		const zip = new JSZip();
		const results = await Promise.allSettled(
			files.map(async (r) => {
				const res = await fetch(rawgithuburl(r));
				if (!res.ok) throw new Error(`failed to fetch ${r.file_path}`);
				zip.file(`${r.owner}-${r.repo}/${r.file_path}`, await res.blob());
			})
		);
		const failed = results.filter((r) => r.status === 'rejected').length;
		if (failed > 0) toast.warning(`${failed} file(s) could not be downloaded`);
		saveblob(await zip.generateAsync({ type: 'blob' }), `${name}.zip`);
	} catch (e) {
		toast.error('download failed');
		console.error(e);
	}
}
