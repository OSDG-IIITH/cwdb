<script lang="ts">
	import { base } from '$app/paths';
	import { Plus, Search } from '@lucide/svelte';
	import { Kbd } from '$lib/components/ui/kbd';
	import { openpalette } from '$lib/components/palette.svelte';
	import { courses, mycourses, syncstatus } from '$lib/sync';
	import CourseCard from '$lib/components/coursecard.svelte';

	const tries = ['dsa mid', 'osn endsem', 'la 2023'];

	let mine = $derived($courses.filter((c) => $mycourses.includes(c.code)));
	let top = $derived([...$courses].sort((a, b) => b.count - a.count).slice(0, 4));
	let empty = $derived($syncstatus.empty && !$syncstatus.syncing);
</script>

<main class="mx-auto flex w-full max-w-180 flex-1 flex-col gap-14 px-6 pt-18 pb-16">
	<div class="flex flex-col items-center gap-7">
		<div class="flex flex-col items-center gap-3">
			<div class="font-mono text-6xl leading-none font-bold tracking-[-0.06em] sm:text-7xl">
				cwdb
			</div>
			<p class="text-sm text-muted-foreground">Papers, slides and notes for every IIIT course</p>
		</div>
		{#if empty}
			<p class="max-w-sm text-center text-sm text-muted-foreground">
				No data loaded yet. Connect to VPN and reload to sync the index.
			</p>
		{:else}
			<div class="flex w-full flex-col gap-2.5">
				<button
					type="button"
					class="flex h-12 w-full items-center gap-2.5 rounded-xl border bg-card pr-3 pl-4 text-left text-[15px] text-muted-foreground shadow-xs transition-all hover:border-foreground/20 hover:shadow-sm"
					onclick={() => openpalette()}
				>
					<Search class="size-4.5" />
					<span class="flex-1 truncate">Search courses, papers, slides</span>
					<Kbd class="hidden border bg-background font-mono sm:inline-flex">Ctrl K</Kbd>
				</button>
				<div
					class="flex flex-wrap items-center justify-center gap-1.5 pt-1 text-xs text-muted-foreground"
				>
					<span class="pr-1">Try</span>
					{#each tries as q (q)}
						<button
							type="button"
							class="rounded-full border px-2.5 py-1 font-mono text-[11px] transition-colors hover:border-foreground/20 hover:text-foreground"
							onclick={() => openpalette(q)}>{q}</button
						>
					{/each}
				</div>
				{#if !$syncstatus.online && !$syncstatus.syncing}
					<p class="text-center text-xs text-muted-foreground">offline · using the saved index</p>
				{/if}
			</div>
		{/if}
	</div>

	{#if !empty}
		<section class="flex flex-col gap-3">
			<div class="flex items-baseline justify-between">
				<h2 class="text-[13px] font-medium">{mine.length ? 'My courses' : 'Largest courses'}</h2>
				<a href="{base}/courses" class="text-[13px] text-muted-foreground hover:text-foreground"
					>All courses</a
				>
			</div>
			<div class="grid grid-cols-[repeat(auto-fit,minmax(min(300px,100%),1fr))] gap-2">
				{#each mine.length ? mine : top as c (c.code)}<CourseCard {c} />{/each}
				<a
					href="{base}/courses"
					class="flex min-h-[86px] items-center justify-center gap-2 rounded-xl border border-dashed px-4 py-3.5 text-[13px] text-muted-foreground transition-colors hover:border-foreground/20 hover:text-foreground"
				>
					<Plus class="size-3.5" />Add a course
				</a>
			</div>
		</section>
	{/if}
</main>

<footer class="p-6 text-center text-xs text-muted-foreground/70">
	made by <a
		href="https://github.com/nuxshed"
		class="text-muted-foreground underline"
		target="_blank"
		rel="noopener">OSDG</a
	>
</footer>
