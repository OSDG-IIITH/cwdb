<script lang="ts">
	import { Filter, Github, Loader2 } from '@lucide/svelte';
	import { MediaQuery, SvelteSet } from 'svelte/reactivity';
	import { untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { base } from '$app/paths';
	import { toast } from 'svelte-sonner';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Sheet from '$lib/components/ui/sheet';
	import EditPanel from '$lib/components/editpanel.svelte';
	import FileList from '$lib/components/filelist.svelte';
	import Preview from '$lib/components/preview.svelte';
	import {
		deleteSource,
		loadreview,
		loadsources,
		reclassify,
		refreshpending,
		editresources,
		refreshSource,
		reviewqueue,
		sourceinfo
	} from '$lib/api';
	import { user } from '$lib/auth';
	import { buildtree, sourcelist } from '$lib/courseview';
	import { allresources, patchresources, syncstatus, trysync } from '$lib/sync';
	import Resize from '$lib/components/resize.svelte';
	import { githuburl } from '$lib/utils';
	import { downloadfile, downloadzip } from '$lib/zip';

	const phone = new MediaQuery('max-width: 1023px');
	const picked = new SvelteSet<string>();
	const toggled = new SvelteSet<string>();

	let query = $state('');
	let editing = $state(false);
	let moving = $state(false);
	let busy = $state('');

	let admin = $derived($user?.role === 'admin');
	let owner = $derived(page.params.owner);
	let repo = $derived(page.params.repo);
	let root = $derived(page.params.path ?? '');
	let info = $derived($sourceinfo.find((s) => s.owner === owner && s.repo === repo));
	let src = $derived(
		sourcelist($allresources).find((s) => s.owner === owner && s.repo === repo) ??
			(admin && info
				? {
						id: String(info.id),
						owner,
						repo,
						branch: info.branch,
						count: 0,
						courses: 0,
						mapped: 0,
						kinded: 0
					}
				: null)
	);
	let sid = $derived(src?.id ?? '');
	let fresh = $derived($reviewqueue.new[sid] ?? 0);
	let queued = $derived($reviewqueue.rows.filter((r) => String(r.source) === sid).length);
	let home = $derived(`${base}/sources/${owner}/${repo}`);

	let files = $derived(
		$allresources.filter(
			(r) =>
				r.source_id === sid &&
				(admin || !r.hidden) &&
				(!root || r.file_path.startsWith(`${root}/`)) &&
				`${r.title} ${r.file_path}`.toLowerCase().includes(query.trim().toLowerCase())
		)
	);
	let tree = $derived(buildtree(files, root, toggled, !!query.trim()));
	let order = $derived(tree.flatMap((t) => (t.r ? [t.r] : [])));

	let previewwidth = $state(480);
	let fid = $derived(page.url.searchParams.get('f'));
	let cur = $derived(fid ? ($allresources.find((r) => r.id === fid) ?? null) : null);
	let chosen = $derived($allresources.filter((r) => picked.has(r.id)));
	let pair = $derived.by(() => {
		if (!cur) return null;
		if (cur.key)
			return cur.keyfor ? ($allresources.find((r) => r.id === cur!.keyfor) ?? null) : null;
		return $allresources.find((r) => r.keyfor === cur!.id) ?? null;
	});

	const pct = (n: number) => (src?.count ? Math.round((n / src.count) * 100) : 0);

	$effect(() => {
		if (admin) loadreview();
	});

	$effect(() => {
		query;
		toggled.clear();
	});

	$effect(() => {
		const r = cur;
		if (!r || query.trim()) return;
		untrack(() => {
			const parts = r.file_path.slice(root ? root.length + 1 : 0).split('/');
			for (let i = 1; i < parts.length; i++) toggled.add(`/${parts.slice(0, i).join('/')}`);
		});
	});

	$effect(() => {
		if (fid) document.querySelector(`[data-row="${fid}"]`)?.scrollIntoView({ block: 'nearest' });
	});

	/** updates query params in place */
	function go(params: Record<string, string | null>) {
		const url = new URL(page.url);
		for (const [k, v] of Object.entries(params)) {
			if (v === null) url.searchParams.delete(k);
			else url.searchParams.set(k, v);
		}
		goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}

	function fold(id: string) {
		if (!toggled.delete(id)) toggled.add(id);
	}

	function step(by: number) {
		if (!order.length) return;
		const i = order.findIndex((r) => r.id === fid);
		go({ f: order[Math.min(order.length - 1, Math.max(0, i < 0 ? 0 : i + by))].id });
	}

	function select(id: string) {
		if (!picked.delete(id)) picked.add(id);
	}

	async function zip() {
		busy = 'zip';
		await downloadzip(chosen, repo ?? 'source');
		busy = '';
	}

	async function hide() {
		const ids = [...picked];
		if (!(await editresources(ids, { hidden: true }))) return toast.error('could not hide');
		patchresources(ids, { hidden: true });
		picked.clear();
	}

	async function sync() {
		busy = 'sync';
		const ok = await refreshSource(Number(sid));
		busy = '';
		if (!ok) return toast.error('sync failed');
		toast.success('synced, publish to update the index');
		loadsources();
		refreshpending();
		trysync();
	}

	async function rules() {
		busy = 'rules';
		const n = await reclassify();
		busy = '';
		if (n === null) return toast.error('reclassify failed');
		toast.success(`${n} files updated, publish to update the index`);
	}

	async function classify() {
		await navigator.clipboard.writeText(
			`cargo run --manifest-path backend/Cargo.toml -- classifyexport ${owner}/${repo}`
		);
		toast.success('export command copied, run it from the repo root');
	}

	async function remove() {
		if (!confirm(`delete ${owner}/${repo}?`)) return;
		if (!(await deleteSource(Number(sid)))) return toast.error('could not delete');
		loadsources();
		goto(`${base}/sources`);
	}

	function onkeydown(e: KeyboardEvent) {
		const el = e.target as HTMLElement;
		if (
			!src ||
			e.ctrlKey ||
			e.metaKey ||
			e.altKey ||
			el.closest('input, textarea, select, [contenteditable]') ||
			document.querySelector('[role="dialog"]')
		)
			return;
		const keys: Record<string, () => void> = {
			j: () => step(1),
			k: () => step(-1),
			' ': () => (cur ? go({ f: null }) : step(0)),
			Escape: () => go({ f: null }),
			x: () => cur && select(cur.id),
			d: () => (picked.size ? zip() : cur && downloadfile(cur)),
			o: () => cur && window.open(githuburl(cur), '_blank'),
			e: () => admin && cur && (editing = true)
		};
		if (e.key in keys) {
			e.preventDefault();
			keys[e.key]();
		}
	}
</script>

<svelte:window {onkeydown} />

{#snippet pane(mobile: boolean)}
	{#if cur}
		<Preview
			r={cur}
			{pair}
			{admin}
			{mobile}
			onclose={() => go({ f: null })}
			onpick={(id) => go({ f: id })}
			onedit={() => (editing = true)}
			onprev={() => step(-1)}
			onnext={() => step(1)}
			ongosrc={() =>
				goto(`${home}/${cur!.file_path.split('/').slice(0, -1).join('/')}?f=${cur!.id}`)}
		/>
	{/if}
{/snippet}

{#if src}
	<section class="relative flex min-w-0 flex-1 flex-col">
		<div class="flex shrink-0 flex-wrap items-center gap-3 border-b px-5 py-3.5 lg:px-6">
			<div class="flex min-w-64 flex-1 flex-col gap-0.5">
				<h1 class="truncate text-base font-semibold tracking-tight">
					<span class="font-normal text-muted-foreground">{owner} /</span>
					{repo}
				</h1>
				<div class="text-xs text-muted-foreground">
					{src.count.toLocaleString()} files · {pct(src.mapped)}% with a course · {pct(src.kinded)}%
					with a kind · {src.branch}{root ? ` · in ${root}` : ''}
				</div>
			</div>
			<div class="flex flex-wrap items-center gap-2">
				{#if admin}
					<span class="rounded-md bg-muted px-1.5 py-0.5 text-[11px] text-muted-foreground"
						>admin</span
					>
					<Button size="sm" variant="outline" disabled={!!busy} onclick={sync}>
						{#if busy === 'sync'}<Loader2 class="animate-spin" />{/if}Sync
					</Button>
					<Button size="sm" variant="outline" disabled={!!busy} onclick={rules}>
						{#if busy === 'rules'}<Loader2 class="animate-spin" />{/if}Reclassify
					</Button>
					{#if fresh}
						<Button size="sm" variant="outline" onclick={classify}>Classify {fresh} new</Button>
					{/if}
					{#if queued}
						<Button size="sm" variant="outline" href="{base}/sources/review">Review {queued}</Button
						>
					{/if}
					<Button size="sm" variant="ghost" class="text-destructive" onclick={remove}>Delete</Button
					>
				{/if}
				<Button
					size="sm"
					variant="outline"
					href="https://github.com/{owner}/{repo}"
					target="_blank"
					rel="noreferrer"
				>
					<Github />GitHub
				</Button>
				<div class="relative">
					<Filter class="absolute top-2 left-2.5 size-3.5 text-muted-foreground" />
					<Input
						class="h-8 w-44 bg-muted/60 pl-8 shadow-none"
						placeholder="Filter"
						aria-label="Filter this source"
						data-slash
						bind:value={query}
					/>
				</div>
			</div>
		</div>

		<FileList
			groups={[]}
			{tree}
			active={fid}
			{picked}
			{admin}
			showkind={false}
			courselinks
			ontoggle={(r) => select(r.id)}
			onopen={(r) => go({ f: r.id })}
			onfold={fold}
		/>

		{#if picked.size}
			<div
				class="absolute bottom-4 left-1/2 flex h-11 -translate-x-1/2 items-center gap-3 rounded-xl bg-foreground pr-2 pl-4 text-[13px] whitespace-nowrap text-background shadow-lg"
			>
				<span><span class="font-mono">{picked.size}</span> selected</span>
				<Button size="sm" variant="secondary" disabled={!!busy} onclick={zip}>Download zip</Button>
				{#if admin}
					<Button
						size="sm"
						variant="ghost"
						class="hover:bg-background/10"
						onclick={() => (moving = true)}>Move to…</Button
					>
					<Button size="sm" variant="ghost" class="hover:bg-background/10" onclick={hide}
						>Hide</Button
					>
				{/if}
				<Button
					size="sm"
					variant="ghost"
					class="text-background/60 hover:bg-background/10"
					onclick={() => picked.clear()}>Clear</Button
				>
			</div>
		{/if}
	</section>

	{#if cur && !phone.current}
		<aside
			class="relative flex w-(--w) shrink-0 flex-col border-l"
			style="--w: {previewwidth}px"
		>
			<Resize
				bind:width={previewwidth}
				min={360}
				max={800}
				fallback={480}
				side="left"
				key="preview"
			/>
			{@render pane(false)}
		</aside>
	{/if}

	{#if phone.current}
		<Sheet.Root open={!!cur} onOpenChange={(o) => !o && go({ f: null })}>
			<Sheet.Content
				side="bottom"
				showCloseButton={false}
				class="gap-0 rounded-none data-[side=bottom]:h-dvh"
			>
				<Sheet.Title class="sr-only">{cur?.title}</Sheet.Title>
				{@render pane(true)}
			</Sheet.Content>
		</Sheet.Root>
	{/if}

	{#if editing && cur}
		<EditPanel ids={[cur.id]} initial={cur} oncancel={() => (editing = false)} />
	{/if}

	{#if moving}
		<EditPanel ids={[...picked]} oncancel={() => (moving = false)} />
	{/if}
{:else if $syncstatus.syncing && !$allresources.length}
	<div class="flex flex-1 items-center justify-center">
		<Loader2 class="size-8 animate-spin text-muted-foreground" />
	</div>
{:else}
	<div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">
		source not found
	</div>
{/if}
