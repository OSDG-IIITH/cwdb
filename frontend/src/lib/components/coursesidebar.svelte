<script lang="ts">
	import {
		BookMarked,
		BookOpen,
		ClipboardList,
		Download,
		FileText,
		Files,
		FlaskConical,
		FolderGit2,
		Loader2,
		NotebookPen,
		Pin,
		Presentation,
		Shapes
	} from '@lucide/svelte';
	import type { Component } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { kinds, type CourseInfo } from '$lib/sync';
	import { cn } from '$lib/utils';

	let {
		course,
		nav,
		counts,
		sources,
		pinned,
		busy,
		onnav,
		onpin,
		ondownload
	}: {
		course: CourseInfo;
		nav: string;
		counts: Record<string, number>;
		sources: { source_id: string; owner: string; repo: string; count: number }[];
		pinned: boolean;
		busy: boolean;
		onnav: (nav: string) => void;
		onpin: () => void;
		ondownload: () => void;
	} = $props();

	const exams = [
		['quiz', 'Quizzes'],
		['mid', 'Mid'],
		['end', 'End'],
		['lab', 'Lab']
	];
	const icons: Record<string, Component> = {
		all: Files,
		exams: FileText,
		assignments: ClipboardList,
		labs: FlaskConical,
		slides: Presentation,
		notes: NotebookPen,
		references: BookOpen,
		projects: FolderGit2,
		other: Shapes
	};
	const title = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
</script>

{#snippet item(key: string, label: string, count: number, indent = false, hint = '')}
	{@const Icon = key.startsWith('src:') ? BookMarked : icons[key]}
	<button
		type="button"
		title={hint || undefined}
		onclick={() => onnav(key)}
		class={cn(
			'flex h-7 items-center gap-2 rounded-md pr-2 text-left text-[13px] text-muted-foreground transition-colors hover:bg-accent/60 hover:text-foreground',
			indent ? 'pl-8' : 'pl-2',
			nav === key && 'bg-accent font-medium text-foreground'
		)}
	>
		{#if Icon}<Icon class="size-3.5 shrink-0" />{/if}
		<span class="flex-1 truncate">{label}</span>
		<span class="text-[11px] text-muted-foreground/70">{count}</span>
	</button>
{/snippet}

<aside class="flex w-full flex-col overflow-y-auto border-r bg-sidebar">
	<div class="flex flex-col gap-1 px-4 pt-4 pb-4">
		<div class="flex items-center gap-1">
			<span class="flex-1 truncate font-mono text-xs text-muted-foreground"
				>{course.alias} · {course.code}</span
			>
			<Button
				variant="ghost"
				size="icon-sm"
				class="size-7 text-muted-foreground hover:text-foreground"
				aria-label={pinned ? 'Remove from my courses' : 'Add to my courses'}
				title={pinned ? 'Remove from my courses' : 'Add to my courses'}
				onclick={onpin}
			>
				<Pin class={pinned ? 'fill-foreground text-foreground' : ''} />
			</Button>
			<Button
				variant="ghost"
				size="icon-sm"
				class="size-7 text-muted-foreground hover:text-foreground"
				aria-label="Download course as zip"
				title="Download course as zip"
				disabled={busy}
				onclick={ondownload}
			>
				{#if busy}<Loader2 class="animate-spin" />{:else}<Download />{/if}
			</Button>
		</div>
		<div class="text-base leading-snug font-semibold tracking-tight">{course.name}</div>
		<div class="text-xs text-muted-foreground">
			{[course.semester && title(course.semester), `${course.count} files`]
				.filter(Boolean)
				.join(' · ')}
		</div>
	</div>

	<nav aria-label="Kinds" class="flex flex-col gap-px px-2">
		<div class="px-2 pt-2 pb-1 text-[11px] font-medium text-muted-foreground/70">Kinds</div>
		{@render item('all', 'All files', counts.all ?? 0)}
		{#each kinds.filter((k) => counts[k]) as k (k)}
			{@render item(k, title(k), counts[k])}
			{#if k === 'exams'}
				{#each exams.filter(([t]) => counts[`exams:${t}`]) as [t, label] (t)}
					{@render item(`exams:${t}`, label, counts[`exams:${t}`], true)}
				{/each}
			{/if}
		{/each}
	</nav>

	{#if sources.length}
		<nav aria-label="Sources" class="flex flex-col gap-px px-2 pb-4">
			<div class="px-2 pt-4 pb-1 text-[11px] font-medium text-muted-foreground/70">Sources</div>
			{#each sources as s (s.source_id)}
				{@render item(`src:${s.source_id}`, s.repo, s.count, false, `${s.owner}/${s.repo}`)}
			{/each}
		</nav>
	{/if}
</aside>
