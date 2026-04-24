<script lang="ts">
	import ResourceExplorer from '$lib/components/resources/resource-explorer.svelte';
	import {
		getcourse,
		searchResources,
		togglecoursepin,
		type Resource,
		type CourseDetail
	} from '$lib/api';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { Loader2, Search, Download, Pin } from '@lucide/svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import JSZip from 'jszip';
	import { makeloadingstate } from '$lib/hooks/loading';
	import { bindslashfocus } from '$lib/hooks/shortcuts';
	import { rawgithuburl, titlecase } from '$lib/utils';

	let course = $state<CourseDetail | null>(null);
	let resources = $state<Resource[]>([]);
	let searchresults = $state<Resource[]>([]);
	let pinned = $state(false);
	let loading = $state(true);
	let showloadingindicator = $state(false);
	let downloading = $state(false);
	let searchquery = $state('');
	let searchinput = $state<HTMLInputElement | null>(null);
	let searching = $state(false);

	let loadingcontrol = makeloadingstate(
		(value) => {
			loading = value;
		},
		(value) => {
			showloadingindicator = value;
		}
	);
	let unbindslash: (() => void) | null = null;
	let debounce: ReturnType<typeof setTimeout> | null = null;

	let slug = $derived(page.params.slug ?? '');
	let path = $derived(page.params.path ? page.params.path.split('/') : []);
	let basepath = $derived(`/courses/${slug}`);
	let abbreviation = $derived((course?.aliases ?? [])[0]?.toUpperCase() ?? '');

	function capseason(s: string): string {
		return s.replace(/\b(monsoon|spring|both)\b/gi, (w) => w.charAt(0).toUpperCase() + w.slice(1));
	}

	async function loadcourse() {
		loadingcontrol.start();
		try {
			const data = await getcourse(slug);
			if (data) {
				course = data.course;
				resources = data.resources;
				pinned = data.pinned;
			}
		} finally {
			loadingcontrol.stop();
		}
	}

	function handlesearch() {
		if (debounce) clearTimeout(debounce);
		debounce = setTimeout(async () => {
			if (!searchquery.trim()) {
				searching = false;
				searchresults = [];
				return;
			}
			searching = true;
			searchresults = await searchResources(searchquery, course?.name);
		}, 200);
	}

	async function handlepin() {
		const result = await togglecoursepin(slug);
		if (result) pinned = result.pinned;
	}

	async function downloadall() {
		if (downloading || resources.length === 0) return;
		downloading = true;
		try {
			const zip = new JSZip();
			const results = await Promise.allSettled(
				resources.map(async (r) => {
					const res = await fetch(rawgithuburl(r));
					if (!res.ok) throw new Error(`failed to fetch ${r.file_path}`);
					const blob = await res.blob();
					zip.file(`${r.owner}-${r.repo}/${r.file_path}`, blob);
				})
			);
			const failed = results.filter((r) => r.status === 'rejected').length;
			if (failed > 0) {
				toast.warning(`${failed} file(s) could not be downloaded`);
			}
			const blob = await zip.generateAsync({ type: 'blob' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `${course?.name ?? 'course'}.zip`;
			a.click();
			URL.revokeObjectURL(url);
		} catch (e) {
			toast.error('download failed');
			console.error(e);
		} finally {
			downloading = false;
		}
	}

	onMount(() => {
		unbindslash = bindslashfocus(() => searchinput);
		loadcourse();
	});

	onDestroy(() => {
		loadingcontrol.destroy();
		if (debounce) clearTimeout(debounce);
		unbindslash?.();
	});
</script>

<div class="flex min-h-screen flex-col bg-background font-sans text-foreground">
	<PageHeader title="Courses" />

	<main class="container mx-auto max-w-6xl flex-1 p-6">
		{#if loading && showloadingindicator}
			<div class="flex items-center justify-center py-20">
				<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
			</div>
		{:else if loading}
			<div class="py-20" aria-hidden="true"></div>
		{:else if course}
			<div class="relative mb-8">
				<Button
					variant="outline"
					size="lg"
					class="group/pin absolute top-0 right-0"
					onclick={handlepin}
				>
					<Pin class={`h-4 w-4 transition-colors ${pinned ? 'fill-foreground' : 'group-hover/pin:fill-foreground/25'}`} />
					{pinned ? 'Unpin' : 'Pin'}
				</Button>

				<div class="flex flex-wrap items-center gap-2">
					{#if course.code}
						<span
							class="rounded-md bg-muted/60 px-2 py-0.5 font-mono text-xs font-medium text-muted-foreground"
						>
							{course.code}
						</span>
					{/if}
					{#if abbreviation}
						<span
							class="rounded-md bg-muted/60 px-2 py-0.5 font-mono text-xs font-medium text-muted-foreground"
						>
							{abbreviation}
						</span>
					{/if}
				</div>

				<h1 class="mt-3 text-4xl leading-tight font-extrabold tracking-tight md:text-5xl">{titlecase(course.name)}</h1>

				<div class="mt-6 flex flex-wrap items-end justify-between gap-4">
					<div class="flex flex-wrap items-start gap-x-8 gap-y-3">
						{#if course.semester || course.year}
							<div>
								<div class="mb-1 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">Season</div>
								<div class="text-sm text-foreground">
									{#if course.semester}{capseason(course.semester)}{/if}{#if course.semester && course.year}{' '}{/if}{#if course.year}{course.year}{/if}
								</div>
							</div>
						{/if}
						{#if course.instructors.length > 0}
							<div>
								<div class="mb-1 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">Instructors</div>
								<div class="text-sm text-foreground">{course.instructors.join(', ')}</div>
							</div>
						{/if}
						<div>
							<div class="mb-1 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">Resources</div>
							<div class="text-sm text-foreground">{resources.length}</div>
						</div>
					</div>

					<Button
						size="lg"
						onclick={downloadall}
						disabled={downloading || resources.length === 0}
					>
						{#if downloading}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							Downloading...
						{:else}
							<Download class="mr-2 h-4 w-4" />
							Download
						{/if}
					</Button>
				</div>
			</div>

			<div class="relative mb-6">
				<Search class="absolute top-2.5 left-2 h-4 w-4 text-muted-foreground" />
				<Input
					placeholder="search resources"
					class="pl-8 focus-visible:ring-0"
					bind:value={searchquery}
					bind:ref={searchinput}
					oninput={handlesearch}
				/>
			</div>

			{#if searching && searchquery.trim()}
				<ul class="space-y-4">
					{#if searchresults.length > 0}
						{#each searchresults as r (r.id)}
							<li>
								<button
									type="button"
									class="w-full cursor-pointer rounded-md border border-border bg-background p-4 text-left transition-colors hover:bg-muted/30"
									onclick={() => window.open(rawgithuburl(r), '_blank')}
								>
									<div class="text-base font-medium text-foreground">{r.title}</div>
									<div class="mt-1 text-xs text-muted-foreground">{r.file_path}</div>
									<div class="mt-3 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
										<span class="rounded bg-muted px-2 py-0.5">{r.owner}/{r.repo}</span>
										{#if r.type}
											<span class="rounded bg-muted px-2 py-0.5">{r.type}</span>
										{/if}
									</div>
								</button>
							</li>
						{/each}
					{:else}
						<li class="py-10 text-center text-sm text-muted-foreground">no results found.</li>
					{/if}
				</ul>
			{:else}
				<ResourceExplorer {resources} {path} {basepath} />
			{/if}
		{:else}
			<div class="py-20 text-center">
				<p class="text-muted-foreground">course not found.</p>
			</div>
		{/if}
	</main>
</div>
