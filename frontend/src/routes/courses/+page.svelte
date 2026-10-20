<script lang="ts">
	import { base } from '$app/paths';
	import { ListFilter } from '@lucide/svelte';
	import { Input } from '$lib/components/ui/input';
	import { Kbd } from '$lib/components/ui/kbd';
	import * as ToggleGroup from '$lib/components/ui/toggle-group';
	import { courses, mycourses, allresources, type CourseInfo } from '$lib/sync';
	import { coursehref } from '$lib/utils';

	const letters = [
		['exams', 'E'],
		['assignments', 'A'],
		['labs', 'L'],
		['slides', 'S'],
		['notes', 'N'],
		['references', 'R'],
		['projects', 'P']
	];
	const terms: Record<string, string> = { monsoon: 'Monsoon', spring: 'Spring', both: 'Both' };
	const grid =
		'grid grid-cols-[56px_minmax(0,1fr)_48px] items-center gap-4 px-3 sm:grid-cols-[64px_minmax(0,1fr)_110px_112px_56px]';

	let filter = $state('');
	let term = $state('all');

	let shown = $derived.by(() => {
		const q = filter.trim().toLowerCase();
		return $courses.filter(
			(c) =>
				(c.count > 0 || $mycourses.includes(c.code)) &&
				(term === 'all' || c.semester === term || c.semester === 'both') &&
				(!q || [c.name, c.code, ...c.shortnames].some((s) => s.toLowerCase().includes(q)))
		);
	});
	let mine = $derived(shown.filter((c) => $mycourses.includes(c.code)));
	let rest = $derived(shown.filter((c) => !$mycourses.includes(c.code)));
	let unclassified = $derived($allresources.filter((r) => !r.course_id).length);
	let showunclassified = $derived(
		term === 'all' && 'unclassified'.includes(filter.trim().toLowerCase())
	);
</script>

{#snippet row(c: CourseInfo)}
	<a
		href={coursehref(c.code)}
		class="{grid} h-11 border-b border-border/50 text-sm hover:bg-muted/40"
	>
		<span class="truncate font-mono text-xs text-muted-foreground">{c.alias}</span>
		<span class="truncate font-medium">{c.name}</span>
		<span class="hidden text-[13px] text-muted-foreground sm:block">{terms[c.semester] ?? '—'}</span
		>
		<span class="hidden font-mono text-xs tracking-[0.25em] sm:block">
			{#each letters as [k, l] (k)}<span class={c.kinds.includes(k) ? '' : 'text-border'}>{l}</span
				>{/each}
		</span>
		<span class="text-right font-mono text-xs text-muted-foreground">{c.count}</span>
	</a>
{/snippet}

<main class="mx-auto flex w-full max-w-260 flex-1 flex-col gap-6 px-4 pt-10 pb-16 sm:px-6">
	<div class="flex flex-wrap items-center gap-3">
		<label class="relative flex flex-[1_1_280px] items-center sm:max-w-90">
			<ListFilter class="pointer-events-none absolute left-3 size-3.5 text-muted-foreground" />
			<Input
				data-slash
				bind:value={filter}
				placeholder="Filter by name, code or alias"
				aria-label="Filter courses"
				class="h-9 pr-9 pl-8 shadow-none"
			/>
			<Kbd class="pointer-events-none absolute right-2 border bg-background font-mono">/</Kbd>
		</label>
		<ToggleGroup.Root
			type="single"
			value={term}
			onValueChange={(v) => (term = v || term)}
			class="rounded-lg bg-muted p-0.5"
		>
			{#each [['all', 'All'], ['monsoon', 'Monsoon'], ['spring', 'Spring']] as [v, label] (v)}
				<ToggleGroup.Item
					value={v}
					class="h-7.5 rounded-md px-3 text-[13px] text-muted-foreground data-[state=on]:bg-background data-[state=on]:font-medium data-[state=on]:text-foreground data-[state=on]:shadow-[0_0_0_1px_var(--border)]"
				>
					{label}
				</ToggleGroup.Item>
			{/each}
		</ToggleGroup.Root>
		<span class="flex-1"></span>
		<span class="font-mono text-xs text-muted-foreground">{shown.length} courses</span>
	</div>

	<div>
		<div class="{grid} h-8 border-b text-xs text-muted-foreground">
			<span>alias</span><span>course</span><span class="hidden sm:block">term</span>
			<span class="hidden sm:block" title="kinds present">has</span><span class="text-right"
				>files</span
			>
		</div>
		{#if mine.length}
			<div class="px-3 pt-4 pb-1.5 text-xs font-semibold">My courses</div>
			{#each mine as c (c.code)}{@render row(c)}{/each}
			<div class="px-3 pt-6 pb-1.5 text-xs font-semibold">All courses</div>
		{/if}
		{#each rest as c (c.code)}{@render row(c)}{/each}
		{#if showunclassified}
			<a
				href="{base}/courses/unclassified"
				class="{grid} h-11 border-b border-border/50 text-sm hover:bg-muted/40"
			>
				<span class="font-mono text-xs text-muted-foreground/70">—</span>
				<span class="font-medium text-muted-foreground">Unclassified</span>
				<span class="hidden sm:block"></span><span class="hidden sm:block"></span>
				<span class="text-right font-mono text-xs text-muted-foreground">{unclassified}</span>
			</a>
		{/if}
		{#if !shown.length && !showunclassified}
			<p class="py-12 text-center text-sm text-muted-foreground">No courses match.</p>
		{/if}
	</div>

	<div class="hidden flex-wrap gap-4 font-mono text-xs text-muted-foreground sm:flex">
		{#each letters as [k, l] (k)}<span>{l} {k}</span>{/each}
	</div>
</main>
