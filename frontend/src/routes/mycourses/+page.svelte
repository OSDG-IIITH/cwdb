<script lang="ts">
	import { base } from '$app/paths';
	import {
		Archive,
		ArchiveRestore,
		ArrowUpRight,
		ChevronRight,
		LogIn,
		Plus,
		X
	} from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import AddCourse from '$lib/components/addcourse.svelte';
	import CourseCard from '$lib/components/coursecard.svelte';
	import CourseMenu from '$lib/components/coursemenu.svelte';
	import { user, loading, login } from '$lib/auth';
	import { buildgroups, type Row } from '$lib/courseview';
	import {
		addcourses,
		archived,
		archivecourses,
		profile,
		progress,
		removecourse,
		setdone,
		type MyCourse
	} from '$lib/profile';
	import {
		allresources,
		almanac,
		courses,
		offeringsfor,
		predecessors,
		syncstatus,
		type CourseInfo,
		type Resource
	} from '$lib/sync';
	import { inphase, phase, shortdate, termkey, termname, termof, termorder } from '$lib/terms';
	import { cn, coursehref } from '$lib/utils';

	const now = new Date();
	const term = termof(now);

	let adding = $state(false);
	let managing = $state(false);
	let shelf = $state(false);

	let p = $derived($profile);
	let info = $derived(new Map($courses.map((c) => [c.code, c])));
	let saved = $derived(
		(p?.courses ?? [])
			.map((m) => ({ m, c: info.get(m.course) }))
			.filter((x): x is { m: MyCourse; c: CourseInfo } => !!x.c)
	);
	let active = $derived(saved.filter(({ m }) => !archived(m, p!.autoterm, now)));
	let past = $derived(saved.filter(({ m }) => archived(m, p!.autoterm, now)));

	/** archived courses by the term they were added in, or in one shelf when terms are off */
	let shelves = $derived.by(() => {
		if (!p?.autoterm) return past.length ? [{ name: 'Archived', list: past }] : [];
		const found = new Map<string, { name: string; order: number; list: typeof past }>();
		for (const x of past) {
			const t = termof(new Date(x.m.added));
			const s = found.get(termkey(t)) ?? { name: termname(t), order: termorder(t), list: [] };
			s.list.push(x);
			found.set(termkey(t), s);
		}
		return [...found.values()].sort((a, b) => b.order - a.order);
	});

	let next = $derived(p?.examfocus ? phase($almanac, now, p.ug1) : null);

	/** each active course's papers for the next exam, newest first */
	let papers = $derived.by(() => {
		const out = new Map<string, Row[]>();
		if (!next) return out;
		for (const { c } of active) {
			const own = $allresources.filter((r) => r.course_id === c.code && !r.hidden);
			const earlier: [string, Resource[]][] = predecessors(c.code).map((code) => [
				code,
				$allresources.filter((r) => r.course_id === code && !r.hidden)
			]);
			const offs = new Map(
				[c.code, ...earlier.map(([code]) => code)].flatMap(offeringsfor).map((o) => [o.id, o])
			);
			const rows = buildgroups(own, earlier, `exams:${next.exam}`, '', offs)
				.flatMap((g) => g.rows)
				.filter((row) => inphase(row.r, next!));
			out.set(c.code, rows);
		}
		return out;
	});

	const added = (m: MyCourse) => shortdate(new Date(m.added));
</script>

{#snippet manage({ m, c }: { m: MyCourse; c: CourseInfo }, old: boolean)}
	<div
		data-code={c.code}
		class="grid h-11 grid-cols-[64px_minmax(0,1fr)_auto] items-center gap-4 rounded-md px-3 text-sm transition-colors hover:bg-accent/60 sm:grid-cols-[80px_minmax(0,1fr)_96px_auto]"
	>
		<span class="truncate font-mono text-xs text-muted-foreground">{c.alias}</span>
		<a href={coursehref(c.code)} class="truncate font-medium hover:underline">{c.name}</a>
		<span class="hidden text-xs text-muted-foreground sm:block">added {added(m)}</span>
		<span class="flex items-center gap-0.5">
			{#if old}
				<Button
					variant="ghost"
					size="icon-sm"
					class="text-muted-foreground"
					aria-label="Restore {c.alias}"
					title="Restore"
					onclick={() => (p?.autoterm ? addcourses([c.code]) : archivecourses([c.code], false))}
					><ArchiveRestore /></Button
				>
			{:else if !p?.autoterm}
				<Button
					variant="ghost"
					size="icon-sm"
					class="text-muted-foreground"
					aria-label="Archive {c.alias}"
					title="Archive"
					onclick={() => archivecourses([c.code], true)}><Archive /></Button
				>
			{/if}
			<Button
				variant="ghost"
				size="icon-sm"
				class="text-muted-foreground"
				aria-label="Remove {c.alias}"
				title="Remove from my courses"
				onclick={() => removecourse(c.code)}><X /></Button
			>
		</span>
	</div>
{/snippet}

{#snippet focus(c: CourseInfo)}
	{@const rows = papers.get(c.code) ?? []}
	{@const done = rows.filter((row) => $progress.get(row.r.id)).length}
	<section data-code={c.code} class="flex flex-col rounded-xl border bg-card shadow-xs">
		<a href={coursehref(c.code)} class="group flex items-center gap-3 px-4 pt-3 pb-2.5">
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="truncate font-mono text-xs text-muted-foreground">{c.alias}</span>
				<span class="truncate text-sm font-medium">{c.name}</span>
			</span>
			{#if rows.length}
				<span class="text-xs text-muted-foreground">{done}/{rows.length} done</span>
			{/if}
			<ArrowUpRight
				class="size-3.5 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100"
				aria-hidden="true"
			/>
		</a>
		{#if rows.length}
			<div class="flex flex-col gap-px border-t px-1.5 py-1.5">
				{#each rows as row (row.r.id)}
					{@const done = !!$progress.get(row.r.id)}
					<div
						class="group flex h-9 items-center gap-3 rounded-md pl-2.5 text-sm transition-colors hover:bg-accent/60"
					>
						<Checkbox
							checked={done}
							onCheckedChange={(v) => setdone(row.r.id, v)}
							aria-label="Mark {row.r.title} as done"
							class={cn(done && 'invisible group-hover:visible pointer-coarse:visible')}
						/>
						<a
							href="{coursehref(c.code)}?in=exams:{next?.exam}&f={row.r.id}"
							class="flex h-full min-w-0 flex-1 items-center gap-3 pr-2.5"
						>
							<span
								class={cn('min-w-0 flex-1 truncate', done && 'text-muted-foreground line-through')}
								>{row.r.title}</span
							>
							{#if row.alts.length}
								<span class="hidden text-xs text-muted-foreground/70 sm:inline"
									>{row.alts.length + 1} sources</span
								>
							{/if}
						</a>
					</div>
				{/each}
			</div>
		{:else}
			<p class="border-t px-4 py-3 text-[13px] text-muted-foreground">
				No {next?.label} papers found
			</p>
		{/if}
	</section>
{/snippet}

<main
	class="mx-auto flex w-full max-w-260 flex-1 flex-col gap-6 px-4 pt-10 pb-[calc(env(safe-area-inset-bottom)+6rem)] sm:px-6 sm:pb-16"
>
	{#if !$user}
		{#if !$loading}
			<div class="flex flex-col items-center gap-3 py-20 text-center">
				<p class="text-sm text-muted-foreground">Log in to keep your courses across devices.</p>
				<Button variant="outline" onclick={login}><LogIn />Log in</Button>
			</div>
		{/if}
	{:else if p}
		<div class="flex flex-wrap items-center gap-3">
			<div class="flex min-w-0 flex-col gap-0.5">
				<h1 class="text-[15px] font-semibold">My courses</h1>
				<span class="text-xs text-muted-foreground">
					{[p.autoterm && termname(term), `${active.length} courses`].filter(Boolean).join(' · ')}
				</span>
			</div>
			<span class="flex-1"></span>
			{#if saved.length}
				<Button
					variant="ghost"
					class="text-muted-foreground"
					aria-pressed={managing}
					onclick={() => (managing = !managing)}>{managing ? 'Done' : 'Manage'}</Button
				>
			{/if}
			<Button variant="outline" onclick={() => (adding = true)}><Plus />Add course</Button>
		</div>

		{#if p.examfocus && active.length && !managing}
			{#if next}
				<div class="flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
					<h2 class="text-[13px] font-medium">Up next: {next.label}</h2>
					<span class="text-xs text-muted-foreground">
						{next.from > now ? `starts ${shortdate(next.from)}, ` : ''}papers shown until {shortdate(
							next.until
						)}{next.term.season !== term.season || next.term.year !== term.year
							? ` · ${termname(next.term)}`
							: ''}{next.guessed ? ' · dates estimated from last year' : ''}
					</span>
				</div>
			{:else}
				<p class="text-xs text-muted-foreground">No exam dates have been published yet.</p>
			{/if}
		{/if}

		<CourseMenu>
			{#if managing}
				<div class="flex flex-col">
					{#each active as x (x.c.code)}{@render manage(x, false)}{/each}
				</div>
			{:else if active.length}
				{#if next}
					<div class="grid items-start gap-3 lg:grid-cols-2">
						{#each active as { c } (c.code)}{@render focus(c)}{/each}
					</div>
				{:else}
					<div class="grid grid-cols-[repeat(auto-fill,minmax(min(280px,100%),1fr))] gap-3">
						{#each active as { c } (c.code)}<CourseCard {c} />{/each}
					</div>
				{/if}
			{:else if !$syncstatus.empty}
				<div
					class="flex flex-col items-center gap-3 rounded-xl border border-dashed px-6 py-14 text-center"
				>
					<p class="text-sm text-muted-foreground">
						{past.length
							? 'Nothing this semester yet.'
							: 'Add the courses you are taking to keep them one click away.'}
					</p>
					<div class="flex gap-2">
						<Button variant="outline" onclick={() => (adding = true)}><Plus />Add course</Button>
						<Button variant="ghost" href="{base}/courses">Browse all</Button>
					</div>
				</div>
			{/if}

			{#if shelves.length}
				<section class="flex flex-col gap-1 pt-6">
					<button
						type="button"
						class="flex h-8 items-center gap-2 self-start rounded-md px-1 text-[13px] font-medium text-muted-foreground transition-colors hover:text-foreground"
						aria-expanded={shelf}
						onclick={() => (shelf = !shelf)}
					>
						<ChevronRight class={cn('size-3.5 transition-transform', shelf && 'rotate-90')} />
						Archived
						<span class="text-xs font-normal text-muted-foreground/70">{past.length}</span>
					</button>
					{#if shelf}
						{#each shelves as s (s.name)}
							{#if p.autoterm}
								<div class="px-3 pt-3 pb-1 text-xs font-medium text-muted-foreground">{s.name}</div>
							{/if}
							{#each s.list as x (x.c.code)}{@render manage(x, true)}{/each}
						{/each}
					{/if}
				</section>
			{/if}
		</CourseMenu>
	{/if}
</main>

<AddCourse bind:open={adding} />
