<script lang="ts">
	import { base } from '$app/paths';
	import { page } from '$app/state';
	import { pushState, replaceState } from '$app/navigation';
	import { MediaQuery } from 'svelte/reactivity';
	import HomeSearch from '$lib/components/homesearch.svelte';
	import { Plus, Search } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Kbd } from '$lib/components/ui/kbd';
	import { openpalette } from '$lib/components/palette.svelte';
	import { courses, mycourses, syncstatus } from '$lib/sync';
	import CourseCard from '$lib/components/coursecard.svelte';

	const tries = ['dsa mid', 'osn endsem', 'la 2023'];

	let mine = $derived($courses.filter((c) => $mycourses.includes(c.code)));
	let top = $derived([...$courses].sort((a, b) => b.count - a.count).slice(0, 6));
	let empty = $derived($syncstatus.empty && !$syncstatus.syncing);

	const phone = new MediaQuery('max-width: 639px');

	let query = $derived(page.state.q ?? page.url.searchParams.get('q') ?? '');

	/** keeps the home search in the url; the first letter adds a history entry so back returns home */
	function setquery(q: string) {
		const url = q ? `?q=${encodeURIComponent(q)}` : base || '/';
		if (q && !query) pushState(url, { q });
		else replaceState(url, { q });
	}
</script>

{#if query && !phone.current}
	<HomeSearch {query} onquery={setquery} />
{:else}
	<main class="mx-auto flex w-full max-w-240 flex-1 flex-col gap-16 px-6 pt-[14vh] pb-16">
		<div class="mx-auto flex w-full max-w-180 flex-col items-center gap-8">
			<div class="flex flex-col items-center gap-3">
				<div class="font-mono text-7xl leading-none font-bold tracking-[-0.06em] sm:text-[104px]">
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
					{#if phone.current}
						<button
							type="button"
							class="flex h-13 w-full items-center gap-2.5 rounded-xl border border-input bg-background pr-3 pl-4 text-left text-base text-muted-foreground transition-colors hover:border-muted-foreground/50"
							onclick={() => openpalette()}
						>
							<Search class="size-4.5" />
							<span class="flex-1 truncate">Search courses, papers, slides</span>
						</button>
					{:else}
						<label class="relative flex items-center">
							<Search class="pointer-events-none absolute left-4 size-4.5 text-muted-foreground" />
							<input
								data-slash
								value=""
								oninput={(e) => setquery(e.currentTarget.value)}
								placeholder="Search courses, papers, slides"
								aria-label="Search"
								autocomplete="off"
								spellcheck="false"
								class="h-13 w-full rounded-xl border border-input bg-background pr-16 pl-11 text-base transition-colors outline-none placeholder:text-muted-foreground hover:border-muted-foreground/50 focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/20"
							/>
							<Kbd class="pointer-events-none absolute right-3 border bg-background font-mono"
								>/</Kbd
							>
						</label>
					{/if}
					<div
						class="flex flex-wrap items-center justify-center gap-1.5 pt-1 text-xs text-muted-foreground"
					>
						<span class="pr-1">Try</span>
						{#each tries as q (q)}
							<button
								type="button"
								class="rounded-full border px-2.5 py-1 font-mono text-[11px] transition-colors hover:border-foreground/20 hover:text-foreground"
								onclick={() => (phone.current ? openpalette(q) : setquery(q))}>{q}</button
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
				<div class="flex items-center justify-between">
					<h2 class="text-[13px] font-medium">{mine.length ? 'My courses' : 'Largest courses'}</h2>
					<div class="flex items-center gap-1">
						<a href="{base}/courses" class="text-[13px] text-muted-foreground hover:text-foreground"
							>All courses</a
						>
						<Button
							variant="ghost"
							size="icon-sm"
							class="size-7 text-muted-foreground"
							href="{base}/courses"
							aria-label="Add a course"
							title="Add a course"><Plus /></Button
						>
					</div>
				</div>
				<div class="grid gap-2.5 sm:grid-cols-2 lg:grid-cols-3">
					{#each mine.length ? mine : top as c (c.code)}<CourseCard {c} small />{/each}
				</div>
			</section>
		{/if}
	</main>

	<footer
		class="p-6 pb-[calc(env(safe-area-inset-bottom)+6rem)] text-center text-xs text-muted-foreground/70 sm:pb-6"
	>
		made by <a
			href="https://github.com/nuxshed"
			class="text-muted-foreground underline"
			target="_blank"
			rel="noopener">OSDG</a
		>
	</footer>
{/if}
