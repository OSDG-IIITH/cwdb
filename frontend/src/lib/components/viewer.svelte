<script lang="ts">
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import { FileText } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { githuburl, rawgithuburl } from '$lib/utils';
	import { downloadfile } from '$lib/zip';
	import type { Resource } from '$lib/sync';
	import type { PDFDocumentLoadingTask, PDFDocumentProxy } from 'pdfjs-dist';

	let { r }: { r: Resource } = $props();

	const images = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg'];
	const texts = [
		'txt',
		'c',
		'h',
		'cpp',
		'hpp',
		'py',
		'js',
		'ts',
		'java',
		'rs',
		'go',
		'sh',
		'json',
		'csv',
		'sql',
		'tex',
		'html',
		'css'
	];

	let ext = $derived(r.name.split('.').pop()?.toLowerCase() ?? '');
	let type = $derived(
		ext === 'pdf'
			? 'pdf'
			: ['md', 'markdown'].includes(ext)
				? 'md'
				: images.includes(ext)
					? 'img'
					: texts.includes(ext)
						? 'text'
						: 'other'
	);
	let big = $derived((r.size ?? 0) > 10 * 1024 * 1024);
	let force = $state(false);
	let go = $derived(!big || force);

	let pages = $state(0);
	let ratio = $state(1.3);
	let text = $state('');
	let failed = $state(false);
	let doc: PDFDocumentProxy | null = null;
	let task: PDFDocumentLoadingTask | null = null;

	let foot = $derived(
		type === 'pdf'
			? `pdf · ${pages} pages`
			: type === 'text'
				? `${ext} · ${text.split('\n').length} lines`
				: type === 'md'
					? 'rendered markdown'
					: ext
	);
	let html = $derived(type === 'md' ? DOMPurify.sanitize(marked.parse(text) as string) : '');

	$effect(() => {
		if (!go || type === 'img' || type === 'other') return;
		let dead = false;
		pages = 0;
		failed = false;
		text = '';
		(async () => {
			const res = await fetch(rawgithuburl(r));
			if (!res.ok) throw new Error(`${res.status}`);
			if (type !== 'pdf') {
				text = (await res.text()).slice(0, 200000);
				return;
			}
			const pdfjs = await import('pdfjs-dist');
			pdfjs.GlobalWorkerOptions.workerSrc = (
				await import('pdfjs-dist/build/pdf.worker.min.mjs?url')
			).default;
			task = pdfjs.getDocument({ data: await res.arrayBuffer() });
			const loaded = await task.promise;
			if (dead) return;
			const first = (await loaded.getPage(1)).getViewport({ scale: 1 });
			ratio = first.height / first.width;
			doc = loaded;
			pages = loaded.numPages;
		})().catch(() => !dead && (failed = true));
		return () => {
			dead = true;
			task?.destroy();
			doc = task = null;
		};
	});

	/** renders a page once it nears the viewport */
	function draw(node: HTMLCanvasElement, n: number) {
		let done = false;
		const watch = new IntersectionObserver(
			async ([e]) => {
				if (!e.isIntersecting || done || !doc) return;
				done = true;
				const p = await doc.getPage(n);
				const scale = (node.clientWidth / p.getViewport({ scale: 1 }).width) * devicePixelRatio;
				const viewport = p.getViewport({ scale });
				node.width = viewport.width;
				node.height = viewport.height;
				await p.render({ canvas: node, canvasContext: node.getContext('2d')!, viewport }).promise;
			},
			{ rootMargin: '600px' }
		);
		watch.observe(node);
		return { destroy: () => watch.disconnect() };
	}
</script>

<div class="flex min-h-0 flex-1 flex-col">
	<div class="flex min-h-0 flex-1 items-start justify-center overflow-auto bg-muted/50 p-4 md:p-5">
		{#if type === 'other' || (big && !force) || failed}
			<div
				class="flex w-full max-w-sm flex-col items-start gap-3.5 rounded-xl border bg-background p-7"
			>
				<FileText class="size-7 text-muted-foreground/60" />
				<div class="text-[15px] font-semibold break-all">{r.name}</div>
				<div class="text-[13px] text-muted-foreground">
					{#if failed}could not load this file{:else if big}this file is {Math.round(
							(r.size ?? 0) / 1048576
						)} MB{:else}no inline preview for this file type{/if}
				</div>
				<div class="flex gap-2">
					<Button onclick={() => downloadfile(r)}>Download</Button>
					{#if big && !force && !failed}
						<Button variant="outline" onclick={() => (force = true)}>Preview anyway</Button>
					{:else}
						<Button variant="outline" href={githuburl(r)} target="_blank" rel="noreferrer"
							>Open on GitHub</Button
						>
					{/if}
				</div>
			</div>
		{:else if type === 'img'}
			<img
				src={rawgithuburl(r)}
				alt={r.title}
				class="max-h-full max-w-full rounded border bg-background object-contain"
			/>
		{:else if type === 'pdf'}
			<div class="flex w-full max-w-2xl flex-col gap-3">
				{#each { length: pages } as _, i (i)}
					<canvas use:draw={i + 1} class="w-full border bg-white" style="aspect-ratio: 1 / {ratio}"
					></canvas>
				{/each}
			</div>
		{:else if type === 'md'}
			<article
				class="w-full max-w-xl rounded-lg border bg-background p-6 text-sm leading-relaxed [&_a]:underline [&_code]:font-mono [&_code]:text-xs [&_h1]:mb-2 [&_h1]:text-xl [&_h1]:font-semibold [&_h2]:mt-4 [&_h2]:mb-1 [&_h2]:text-base [&_h2]:font-semibold [&_li]:ml-5 [&_li]:list-disc [&_p]:my-2 [&_pre]:overflow-x-auto [&_pre]:rounded [&_pre]:bg-muted [&_pre]:p-3"
			>
				{@html html}
			</article>
		{:else}
			<pre
				class="w-full max-w-xl overflow-x-auto rounded-lg border bg-background p-5 font-mono text-xs leading-relaxed">{text}</pre>
		{/if}
	</div>
	<div
		class="flex h-9 shrink-0 items-center justify-center border-t bg-background font-mono text-xs text-muted-foreground"
	>
		{foot}
	</div>
</div>
