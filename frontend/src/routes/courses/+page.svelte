<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { Loader2, Search, FileText } from '@lucide/svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import CourseCard from '$lib/components/courses/course-card.svelte';
	import { Input } from '$lib/components/ui/input';
	import * as Pagination from '$lib/components/ui/pagination';
	import { bindslashfocus } from '$lib/hooks/shortcuts';
	import { getcourses, getunclassified, allresources, type CourseInfo } from '$lib/sync';

	const PER_PAGE = 30;

	let searchquery = $state('');
	let searchinput = $state<HTMLInputElement | null>(null);
	let currentpage = $state(1);
	let unbindslash: (() => void) | null = null;
	let debounce: ReturnType<typeof setTimeout> | null = null;

	let allcourses = $derived.by(() => {
		allresources;
		return getcourses();
	});

	let unclassifiedcount = $derived.by(() => {
		allresources;
		return getunclassified().length;
	});

	let filtered = $derived.by(() => {
		const q = searchquery.toLowerCase();
		if (!q) return allcourses;
		return allcourses.filter(
			(c) =>
				c.name.toLowerCase().includes(q) ||
				c.code.toLowerCase().includes(q) ||
				c.aliases.some((a) => a.toLowerCase().includes(q))
		);
	});

	let pinned = $derived(
		filtered.filter((c) => ispinned(c.id)).slice(0, currentpage === 1 ? undefined : 0)
	);
	let unpinned = $derived(filtered.filter((c) => !ispinned(c.id)));
	let total = $derived(unpinned.length);
	let totalpages = $derived(Math.ceil(total / PER_PAGE));
	let pagecourses = $derived(unpinned.slice((currentpage - 1) * PER_PAGE, currentpage * PER_PAGE));
	let empty = $derived(filtered.length === 0);

	function ispinned(id: string): boolean {
		if (typeof localStorage === 'undefined') return false;
		const pins: string[] = JSON.parse(localStorage.getItem('pins') ?? '[]');
		return pins.includes(id);
	}

	function handlesearch() {
		if (debounce) clearTimeout(debounce);
		debounce = setTimeout(() => {
			currentpage = 1;
		}, 200);
	}

	onDestroy(() => {
		if (debounce) clearTimeout(debounce);
		unbindslash?.();
	});

	onMount(() => {
		unbindslash = bindslashfocus(() => searchinput);
	});
</script>

<div class="flex min-h-screen flex-col bg-background font-sans text-foreground">
	<PageHeader title="Courses" />

	<main class="container mx-auto max-w-6xl flex-1 p-6">
		<div class="mb-8 flex items-center justify-between gap-4">
			<div class="relative w-full md:w-64">
				<Search class="absolute top-2.5 left-2 h-4 w-4 text-muted-foreground" />
				<Input
					placeholder="search courses"
					class="pl-8 focus-visible:ring-0"
					bind:value={searchquery}
					bind:ref={searchinput}
					oninput={handlesearch}
				/>
			</div>

			<p class="text-sm whitespace-nowrap text-muted-foreground">
				{total + pinned.length}
				{total + pinned.length === 1 ? 'course' : 'courses'}
			</p>
		</div>

		{#if empty && !searchquery}
			<div class="py-20 text-center">
				<p class="text-muted-foreground">no courses found.</p>
			</div>
		{:else if empty}
			<div class="py-20 text-center">
				<p class="text-muted-foreground">no matching courses found.</p>
			</div>
		{:else}
			{#if pinned.length > 0 && currentpage === 1}
				<div class="mb-8">
					<p class="mb-3 text-xs font-medium tracking-wider text-muted-foreground uppercase">
						Pinned
					</p>
					<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
						{#each pinned as course (course.id)}
							<CourseCard {course} onpintoggle={() => {}} />
						{/each}
					</div>
				</div>
			{/if}

			{#if pagecourses.length > 0}
				{#if pinned.length > 0 && currentpage === 1}
					<p class="mb-3 text-xs font-medium tracking-wider text-muted-foreground uppercase">
						All Courses
					</p>
				{/if}
				<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
					{#each pagecourses as course (course.id)}
						<CourseCard {course} onpintoggle={() => {}} />
					{/each}
				</div>
			{/if}

			{#if unclassifiedcount > 0 && !searchquery}
				<div class="mt-8">
					<p class="mb-3 text-xs font-medium tracking-wider text-muted-foreground uppercase">
						Other
					</p>
					<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
						<a href="/courses/unclassified" class="block">
							<div
								class="group relative flex w-full flex-col justify-between gap-4 rounded-xl border border-border/60 bg-card/40 px-4 py-4 transition-colors hover:border-border md:px-5 md:py-5"
							>
								<div class="space-y-1">
									<span class="font-mono text-xs font-medium text-muted-foreground">N/A</span>
									<p class="truncate text-sm font-medium tracking-tight">Unclassified</p>
								</div>
								<div class="flex items-center gap-1 text-[11px] text-muted-foreground">
									<FileText class="h-3 w-3" />
									<span
										>{unclassifiedcount}
										{unclassifiedcount === 1 ? 'resource' : 'resources'}</span
									>
								</div>
							</div>
						</a>
					</div>
				</div>
			{/if}

			{#if totalpages > 1}
				<div class="mt-8">
					<Pagination.Root
						count={total}
						perPage={PER_PAGE}
						bind:page={currentpage}
					>
						{#snippet children({ pages })}
							<Pagination.Content>
								<Pagination.Item>
									<Pagination.PrevButton />
								</Pagination.Item>
								{#each pages as p (p.key)}
									{#if p.type === 'ellipsis'}
										<Pagination.Item>
											<Pagination.Ellipsis />
										</Pagination.Item>
									{:else}
										<Pagination.Item>
											<Pagination.Link page={p} isActive={currentpage === p.value}>
												{p.value}
											</Pagination.Link>
										</Pagination.Item>
									{/if}
								{/each}
								<Pagination.Item>
									<Pagination.NextButton />
								</Pagination.Item>
							</Pagination.Content>
						{/snippet}
					</Pagination.Root>
				</div>
			{/if}
		{/if}
	</main>
</div>
