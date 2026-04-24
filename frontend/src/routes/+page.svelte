<script lang="ts">
	import { onMount } from 'svelte';
	import { searchResources, listcourses, type Resource, type Course } from '$lib/api';
	import { Input } from '$lib/components/ui/input';
	import CourseFilter from '$lib/components/coursefilter.svelte';
	import { rawgithuburl } from '$lib/utils';
	import { bindslashfocus } from '$lib/hooks/shortcuts';
	import { onDestroy } from 'svelte';

	let query = $state('');
	let results = $state<Resource[]>([]);
	let courses = $state<Course[]>([]);
	let selectedcourse = $state<string | null>(null);

	async function search() {
		results = await searchResources(query, selectedcourse ?? undefined);
	}

	$effect(() => {
		selectedcourse;
		search();
	});

	onMount(async () => {
		const res = await listcourses({ per_page: 100 });
		courses = [...res.pinned, ...res.courses];
	});

	function repoUrl(r: Resource) {
		return `https://github.com/${r.owner}/${r.repo}`;
	}

	function openRaw(r: Resource) {
		window.open(rawgithuburl(r), '_blank');
	}

	let active = $derived(query.length > 0 || selectedcourse !== null);
	let searchInput: HTMLInputElement | null = $state(null);
	let unbindslash: (() => void) | null = null;

	onMount(() => {
		unbindslash = bindslashfocus(() => searchInput);
	});

	onDestroy(() => {
		unbindslash?.();
	});
</script>

<div class="relative mx-auto flex min-h-screen max-w-3xl flex-col justify-between px-4">
	<!-- logo + searchbar -->
	<div
		class={active
			? 'fixed top-0 right-0 left-0 z-40 border-b border-border bg-background'
			: 'flex flex-1 flex-col items-center justify-center text-center'}
	>
		<div
			class={`w-full ${
				active
					? 'mx-auto flex max-w-3xl flex-col gap-4 px-4 py-4 md:flex-row md:items-center md:justify-between'
					: 'flex flex-col items-center gap-4'
			}`}
		>
			<!-- logo -->
			<div
				class={`font-mono font-bold tracking-tight text-foreground ${
					active ? 'text-3xl' : 'text-8xl'
				}`}
			>
				cwdb
			</div>

			<div class={`w-full ${active ? 'mt-0' : 'mt-2'} flex items-stretch gap-2`}>
				<div class="relative flex-1">
					<svg
						class="pointer-events-none absolute top-1/2 left-3 z-10 h-5 w-5 -translate-y-1/2 text-muted-foreground"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						viewBox="0 0 24 24"
						aria-hidden="true"
					>
						<circle cx="11" cy="11" r="7" stroke="currentColor" />
						<line x1="16.65" y1="16.65" x2="21" y2="21" stroke="currentColor" />
					</svg>
					<Input
						bind:ref={searchInput}
						type="text"
						class="h-12 w-full border-border pr-4 pl-10 focus-visible:border-border focus-visible:ring-0"
						placeholder="search"
						bind:value={query}
						oninput={search}
					/>
				</div>
				{#if active}
					<div class="hidden md:block">
						<CourseFilter {courses} bind:selected={selectedcourse} />
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- results -->
	{#if active}
		<div class="mt-32 flex-1 sm:mt-20">
			<ul class="mt-4 mb-10 space-y-4">
				{#if results.length > 0}
					{#each results as r (r.id)}
						<li>
							<button
								type="button"
								class="w-full cursor-pointer rounded-md border border-border bg-background p-4 text-left transition-colors hover:bg-muted/30"
								onclick={() => openRaw(r)}
							>
								<div class="text-base font-medium text-foreground">{r.title}</div>
								<div class="mt-1 text-xs text-muted-foreground">{r.file_path}</div>

								<div class="mt-3 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
									<a
										href={repoUrl(r)}
										target="_blank"
										rel="noopener"
										class="rounded bg-muted px-2 py-0.5 hover:underline"
										onclick={(e: MouseEvent) => e.stopPropagation()}
									>
										{r.owner}/{r.repo}
									</a>

									{#if r.type}
										<span class="rounded bg-muted px-2 py-0.5">
											{r.type}
										</span>
									{/if}

									{#if r.course_name}
										<span class="rounded bg-muted px-2 py-0.5">
											{r.course_name.replace(/\b\w/g, (c) => c.toUpperCase())}
										</span>
									{/if}
								</div>
							</button>
						</li>
					{/each}
				{:else}
					<li class="text-center text-sm text-muted-foreground">No results found.</li>
				{/if}
			</ul>
		</div>
	{/if}

	<!-- footer -->
	<footer class="mt-10 hidden py-6 text-center text-sm text-muted-foreground md:block">
		Made with ♥ by
		<a
			href="https://github.com/nuxshed"
			class="underline hover:opacity-80"
			target="_blank"
			rel="noopener"
		>
			Advait Dintakurti
		</a>
	</footer>
</div>
