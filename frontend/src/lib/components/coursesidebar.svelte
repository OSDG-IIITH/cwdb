<script lang="ts">
	import { Download, FolderClosed, Loader2, Pin } from '@lucide/svelte';
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
	const title = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
</script>

{#snippet item(key: string, label: string, count: number, indent = false, folder = false)}
	<button
		type="button"
		onclick={() => onnav(key)}
		class={cn(
			'flex h-[30px] items-center gap-2 rounded-md pr-2 text-left text-[13px] text-foreground/80 hover:bg-accent',
			indent ? 'pl-6' : 'pl-2',
			nav === key && 'bg-accent font-medium text-foreground'
		)}
	>
		{#if folder}<FolderClosed class="size-3.5 shrink-0 opacity-60" />{/if}
		<span class="flex-1 truncate">{label}</span>
		<span class="font-mono text-[11px] text-muted-foreground/70">{count}</span>
	</button>
{/snippet}

<aside class="flex w-64 shrink-0 flex-col overflow-y-auto border-r bg-muted/40">
	<div class="flex flex-col gap-1.5 px-4 pt-5 pb-4">
		<div class="font-mono text-xs text-muted-foreground">{course.alias} · {course.code}</div>
		<div class="text-[17px] leading-tight font-semibold tracking-tight">{course.name}</div>
		<div class="text-xs text-muted-foreground">
			{[course.semester && title(course.semester), `${course.count} files`]
				.filter(Boolean)
				.join(' · ')}
		</div>
		<div class="mt-2 flex gap-1.5">
			<Button variant="outline" size="sm" class="flex-1 bg-background" onclick={onpin}>
				<Pin class={pinned ? 'fill-foreground' : ''} />
				{pinned ? 'In my courses' : 'Add to my courses'}
			</Button>
			<Button
				variant="outline"
				size="icon-sm"
				class="bg-background"
				aria-label="Download course as zip"
				disabled={busy}
				onclick={ondownload}
			>
				{#if busy}<Loader2 class="animate-spin" />{:else}<Download />{/if}
			</Button>
		</div>
	</div>

	<nav aria-label="Kinds" class="flex flex-col gap-px px-2">
		<div class="px-2 pt-2 pb-1 text-[11px] text-muted-foreground/70">Kinds</div>
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
			<div class="px-2 pt-3 pb-1 text-[11px] text-muted-foreground/70">Sources</div>
			{#each sources as s (s.source_id)}
				{@render item(`src:${s.source_id}`, `${s.owner}/${s.repo}`, s.count, false, true)}
			{/each}
		</nav>
	{/if}
</aside>
