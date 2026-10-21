<script lang="ts">
	import { Filter, Pin } from '@lucide/svelte';
	import { MediaQuery, SvelteSet } from 'svelte/reactivity';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { toast } from 'svelte-sonner';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Kbd } from '$lib/components/ui/kbd';
	import * as Sheet from '$lib/components/ui/sheet';
	import CourseSidebar from '$lib/components/coursesidebar.svelte';
	import EditPanel from '$lib/components/editpanel.svelte';
	import FileList from '$lib/components/filelist.svelte';
	import Preview from '$lib/components/preview.svelte';
	import { editresources } from '$lib/api';
	import { openpalette } from '$lib/components/palette.svelte';
	import { user } from '$lib/auth';
	import Resize from '$lib/components/resize.svelte';
	import Segmented from '$lib/components/segmented.svelte';
	import { buildgroups, buildtree, navcounts } from '$lib/courseview';
	import {
		allresources,
		courses,
		coursefolders,
		getcourse,
		mycourses,
		offeringsfor,
		patchresources,
		predecessors,
		togglepin,
		kinds,
		type CourseInfo,
		type Resource
	} from '$lib/sync';
	import { githuburl } from '$lib/utils';
	import { downloadfile, downloadzip } from '$lib/zip';

	const phone = new MediaQuery('max-width: 1023px');
	const picked = new SvelteSet<string>();
	const closed = new SvelteSet<string>();

	let query = $state('');
	let sidebar = $state(true);
	let editing = $state<'one' | 'many' | null>(null);
	let busy = $state(false);

	let admin = $derived($user?.role === 'admin');
	let course = $derived<CourseInfo | null>(
		page.params.slug === 'unclassified'
			? {
					code: '',
					name: 'Unclassified',
					shortnames: [],
					alias: '',
					semester: '',
					count: 0,
					kinds: []
				}
			: getcourse($courses, page.params.slug)
	);
	let code = $derived(course?.code ?? '');
	let pinned = $derived(!!code && $mycourses.includes(code));

	const shown = (r: Resource) => admin || !r.hidden;

	let own = $derived(
		course
			? $allresources.filter((r) => (code ? r.course_id === code : !r.course_id) && shown(r))
			: []
	);
	let earlier = $derived<[string, Resource[]][]>(
		predecessors(code).map((p) => [p, $allresources.filter((r) => r.course_id === p && shown(r))])
	);
	let offs = $derived(
		new Map(
			[code, ...earlier.map(([p]) => p)].flatMap((c) => offeringsfor(c)).map((o) => [o.id, o])
		)
	);
	let counts = $derived(navcounts([...own, ...earlier.flatMap(([, list]) => list)]));
	let sources = $derived.by(() => {
		void $allresources;
		return code ? coursefolders(code) : [];
	});

	let nav = $derived(
		page.url.searchParams.get('in') ??
			(sources.length && own.filter((r) => !r.kind).length > own.length / 2
				? `src:${sources[0].source_id}`
				: 'all')
	);
	let fid = $derived(page.url.searchParams.get('f'));
	let insource = $derived(nav.startsWith('src:'));
	let by = $state('sem');
	let previewwidth = $state(480);
	let navwidth = $state(256);

	let groups = $derived(insource ? [] : buildgroups(own, earlier, nav, query, offs, by));
	let tree = $derived(
		insource
			? buildtree(
					own.filter(
						(r) =>
							r.source_id === nav.slice(4) &&
							`${r.title} ${r.file_path}`.toLowerCase().includes(query.trim().toLowerCase())
					),
					sources.find((s) => s.source_id === nav.slice(4))?.path ?? '',
					closed
				)
			: null
	);
	let order = $derived(
		tree ? tree.flatMap((t) => (t.r ? [t.r] : [])) : groups.flatMap((g) => g.rows.map((x) => x.r))
	);

	let byid = $derived(new Map($allresources.map((r) => [r.id, r])));
	let cur = $derived(fid ? (byid.get(fid) ?? null) : null);
	let pair = $derived.by(() => {
		if (!cur) return null;
		if (cur.key) return cur.keyfor ? (byid.get(cur.keyfor) ?? null) : null;
		return $allresources.find((r) => r.keyfor === cur!.id) ?? null;
	});

	let title = $derived(
		insource
			? (sources.find((s) => s.source_id === nav.slice(4))?.repo ?? '')
			: nav === 'all'
				? 'All files'
				: nav.includes(':')
					? `Exams · ${nav.split(':')[1]}`
					: nav.charAt(0).toUpperCase() + nav.slice(1)
	);
	let total = $derived(tree ? order.length : groups.reduce((n, g) => n + g.rows.length, 0));

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

	function open(r: Resource) {
		go({ f: r.id });
	}

	function toggle(r: Resource) {
		if (!picked.delete(r.id)) picked.add(r.id);
	}

	function fold(id: string) {
		if (!closed.delete(id)) closed.add(id);
	}

	function step(by: number) {
		if (!order.length) return;
		const i = order.findIndex((r) => r.id === fid);
		open(order[Math.min(order.length - 1, Math.max(0, i < 0 ? 0 : i + by))]);
	}

	/** expands units to every file inside their folder */
	function withunits(list: Resource[]): Resource[] {
		const out = new Map<string, Resource>();
		for (const r of list) {
			out.set(r.id, r);
			if (r.unit) {
				for (const x of own) {
					if (x.source_id === r.source_id && x.file_path.startsWith(`${r.unit}/`)) out.set(x.id, x);
				}
			}
		}
		return [...out.values()];
	}

	async function zip(list: Resource[]) {
		if (busy || !list.length) return;
		busy = true;
		await downloadzip(withunits(list), course?.alias || 'course');
		busy = false;
	}

	let chosen = $derived([...picked].map((id) => byid.get(id)).filter((r) => !!r));

	async function hide() {
		const ids = [...picked];
		if (!(await editresources(ids, { hidden: true }))) return toast.error('could not hide');
		patchresources(ids, { hidden: true });
		picked.clear();
	}

	function onkeydown(e: KeyboardEvent) {
		const el = e.target as HTMLElement;
		if (
			!course ||
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
			x: () => cur && toggle(cur),
			d: () => (picked.size ? zip(chosen) : cur && downloadfile(cur)),
			'[': () => (sidebar = false),
			']': () => (sidebar = true),
			o: () => cur && window.open(githuburl(cur), '_blank'),
			p: () => code && togglepin(code),
			e: () => admin && cur && (editing = 'one')
		};
		if (e.key in keys) {
			e.preventDefault();
			keys[e.key]();
		}
	}
</script>

<svelte:window {onkeydown} />

{#if course}
	<div class="flex h-[calc(100dvh-3rem)] overflow-hidden bg-background text-foreground">
		{#if !phone.current && sidebar && code}
			<div class="relative flex shrink-0" style="width: {navwidth}px">
				<CourseSidebar
					{course}
					{nav}
					{counts}
					{sources}
					{pinned}
					{busy}
					onnav={(n) => go({ in: n })}
					onpin={() => togglepin(code)}
					ondownload={() => zip(own)}
				/>
				<Resize
					bind:width={navwidth}
					min={220}
					max={420}
					fallback={256}
					side="right"
					key="coursenav"
				/>
			</div>
		{/if}

		<section class="relative flex min-w-0 flex-1 flex-col">
			<div class="shrink-0 lg:hidden">
				<div class="flex items-start gap-2 px-5 pt-3 pb-4">
					<div class="flex min-w-0 flex-1 flex-col gap-1.5">
						<div class="font-mono text-xs text-muted-foreground">
							{course.alias} · {course.code}
						</div>
						<div class="text-2xl leading-tight font-semibold tracking-tight">{course.name}</div>
					</div>
					{#if code}
						<Button
							variant="ghost"
							size="icon"
							class="size-11"
							aria-label="Pin course"
							onclick={() => togglepin(code)}
						>
							<Pin class={pinned ? 'fill-foreground' : ''} />
						</Button>
					{/if}
				</div>
				<div class="flex gap-2 overflow-x-auto border-b px-5 pb-3.5 [scrollbar-width:none]">
					{#each ['all', ...kinds.filter((k) => counts[k])] as k (k)}
						<Button
							variant={nav === k ? 'default' : 'outline'}
							class="h-8 shrink-0 rounded-full px-3 text-[13px] capitalize"
							onclick={() => go({ in: k })}
						>
							{k}
							{counts[k] ?? 0}
						</Button>
					{/each}
				</div>
			</div>

			<div class="hidden h-[52px] shrink-0 items-center gap-3 border-b pr-4 pl-6 lg:flex">
				<span class="text-[15px] font-semibold whitespace-nowrap">{title}</span>
				<span class="min-w-0 flex-1 truncate font-mono text-xs text-muted-foreground/70"
					>{total} files</span
				>
				<div class="relative">
					<Filter class="absolute top-2.5 left-2 size-3.5 text-muted-foreground" />
					<Input
						class="h-[30px] w-40 pl-7"
						placeholder="Filter"
						aria-label="Filter this list"
						data-slash
						bind:value={query}
					/>
				</div>
			</div>

			{#if nav === 'all'}
				<div class="shrink-0 px-5 pt-3 lg:px-6">
					<Segmented
						bind:value={by}
						label="Group files"
						options={[
							{ value: 'sem', label: 'By Semester' },
							{ value: 'kind', label: 'By Kind' }
						]}
					/>
				</div>
			{/if}

			<FileList
				{groups}
				{tree}
				active={fid}
				{picked}
				{admin}
				showkind={nav === 'all'}
				ontoggle={toggle}
				onopen={open}
				onfold={fold}
			/>

			<div
				class="hidden h-9 shrink-0 items-center gap-4 border-t px-6 text-xs whitespace-nowrap text-muted-foreground lg:flex"
			>
				<span><Kbd>j</Kbd> <Kbd>k</Kbd> move</span>
				<span><Kbd>space</Kbd> preview</span>
				<span><Kbd>x</Kbd> select</span>
				<span><Kbd>[</Kbd> <Kbd>]</Kbd> sidebar</span>
			</div>

			{#if picked.size}
				<div
					class="absolute bottom-14 left-6 hidden h-11 items-center gap-3 rounded-xl bg-foreground pr-2 pl-4 text-[13px] whitespace-nowrap text-background lg:flex"
				>
					<span><span class="font-mono">{picked.size}</span> selected</span>
					<span class="h-4 w-px bg-background/30"></span>
					<Button size="sm" variant="secondary" disabled={busy} onclick={() => zip(chosen)}>
						Download zip <span class="font-mono text-[11px] text-muted-foreground">d</span>
					</Button>
					{#if admin}
						<Button
							size="sm"
							variant="ghost"
							class="hover:bg-background/10"
							onclick={() => (editing = 'many')}>Move to…</Button
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

			<div
				class="absolute inset-x-0 bottom-0 bg-linear-to-b from-transparent to-background to-30% px-4 pt-3 pb-8 lg:hidden"
			>
				<Input
					class="h-12 rounded-full px-5 text-[15px]"
					placeholder="Search in {course.alias || 'course'}"
					aria-label="Search in this course"
					bind:value={query}
				/>
			</div>
		</section>

		{#if cur && !phone.current}
			<aside
				class="relative flex w-(--w) shrink-0 flex-col border-l bg-muted/50"
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
				<Preview
					r={cur}
					{pair}
					{admin}
					onclose={() => go({ f: null })}
					onpick={(id) => go({ f: id })}
					onedit={() => (editing = 'one')}
					onprev={() => step(-1)}
					onnext={() => step(1)}
					ongosrc={() => go({ in: `src:${cur!.source_id}` })}
				/>
			</aside>
		{/if}
	</div>

	{#if phone.current}
		<Sheet.Root open={!!cur} onOpenChange={(o) => !o && go({ f: null })}>
			<Sheet.Content
				side="bottom"
				showCloseButton={false}
				class="gap-0 rounded-none data-[side=bottom]:h-dvh"
			>
				<Sheet.Title class="sr-only">{cur?.title}</Sheet.Title>
				{#if cur}
					<Preview
						r={cur}
						{pair}
						{admin}
						mobile
						onclose={() => go({ f: null })}
						onpick={(id) => go({ f: id })}
						onedit={() => (editing = 'one')}
						onprev={() => step(-1)}
						onnext={() => step(1)}
						ongosrc={() => go({ in: `src:${cur!.source_id}`, f: null })}
					/>
				{/if}
			</Sheet.Content>
		</Sheet.Root>
	{/if}

	{#if editing}
		<EditPanel
			ids={editing === 'one' && cur ? [cur.id] : [...picked]}
			initial={editing === 'one' ? cur : null}
			oncancel={() => (editing = null)}
		/>
	{/if}
{:else if $courses.length}
	<div class="py-20 text-center text-muted-foreground">
		course not found.
		<Button variant="link" onclick={() => openpalette(page.params.slug)}>search for it</Button>
	</div>
{/if}
