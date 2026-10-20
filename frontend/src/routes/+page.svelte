<script lang="ts">
	import { base } from '$app/paths';
	import { Plus, Search } from '@lucide/svelte';
	import { Kbd } from '$lib/components/ui/kbd';
	import { openpalette } from '$lib/components/palette.svelte';
	import { courses, mycourses, syncstatus } from '$lib/sync';
	import { coursehref } from '$lib/utils';

	const tries = ['dsa mid', 'osn endsem', 'la 2023'];

	let mine = $derived($courses.filter((c) => $mycourses.includes(c.code)));
	let empty = $derived($syncstatus.empty && !$syncstatus.syncing);
</script>

<main class="mx-auto flex w-full max-w-180 flex-1 flex-col gap-14 px-6 pt-18 pb-16">
	<div class="flex flex-col items-center gap-7">
		<div class="font-mono text-7xl leading-none font-bold tracking-[-0.06em] sm:text-[88px]">
			cwdb
		</div>
		{#if empty}
			<p class="max-w-sm text-center text-sm text-muted-foreground">
				No data loaded yet. Connect to VPN and reload to sync the index.
			</p>
		{:else}
			<div class="flex w-full flex-col gap-2.5">
				<button
					type="button"
					class="flex h-13 w-full items-center gap-2.5 rounded-xl border border-input bg-background pr-3 pl-4 text-left text-base text-muted-foreground transition-colors hover:border-muted-foreground/50"
					onclick={() => openpalette()}
				>
					<Search class="size-4.5" />
					<span class="flex-1 truncate">Search courses, papers, slides</span>
					<Kbd class="hidden border bg-background font-mono sm:inline-flex">Ctrl K</Kbd>
				</button>
				<div
					class="flex flex-wrap justify-center gap-x-3.5 gap-y-1.5 text-[13px] text-muted-foreground"
				>
					<span>try</span>
					{#each tries as q (q)}
						<button
							type="button"
							class="font-mono text-xs hover:text-foreground"
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
				<h2 class="text-[13px] font-semibold">My courses</h2>
				<a href="{base}/courses" class="text-[13px] text-muted-foreground hover:text-foreground"
					>All courses</a
				>
			</div>
			<div class="grid grid-cols-[repeat(auto-fit,minmax(min(300px,100%),1fr))] gap-2">
				{#each mine as c (c.code)}
					<a
						href={coursehref(c.code)}
						class="flex flex-col gap-1 rounded-[10px] border px-4 py-3.5 transition-colors hover:border-muted-foreground/50"
					>
						<span class="flex justify-between font-mono text-xs text-muted-foreground">
							<span>{c.alias}</span><span>{c.count}</span>
						</span>
						<span class="text-sm font-medium">{c.name}</span>
					</a>
				{/each}
				<a
					href="{base}/courses"
					class="flex items-center justify-center gap-2 rounded-[10px] border border-dashed border-input px-4 py-3.5 text-[13px] text-muted-foreground transition-colors hover:border-muted-foreground/50 hover:text-foreground"
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
