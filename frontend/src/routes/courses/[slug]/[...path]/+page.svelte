<script lang="ts">
	import ResourceExplorer from '$lib/components/resources/resource-explorer.svelte';
	import { getcourse, searchResources, type Resource, type CourseDetail } from '$lib/api';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { Loader2, Search, Download } from '@lucide/svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import JSZip from 'jszip';

	let course = $state<CourseDetail | null>(null);
	let resources = $state<Resource[]>([]);
	let searchresults = $state<Resource[]>([]);
	let loading = $state(true);
	let showloadingindicator = $state(false);
	let downloading = $state(false);
	let searchquery = $state('');
	let searchinput = $state<HTMLInputElement | null>(null);
	let searching = $state(false);

	const LOADING_DELAY_MS = 250;
	let loadingtimer: ReturnType<typeof setTimeout> | null = null;
	let debounce: ReturnType<typeof setTimeout> | null = null;

	let slug = $derived(page.params.slug ?? '');
	let path = $derived(page.params.path ? page.params.path.split('/') : []);
	let basepath = $derived(`/courses/${slug}`);
	let abbreviation = $derived((course?.aliases ?? [])[0]?.toUpperCase() ?? '');

	function titlecase(str: string): string {
		return str.replace(/\b\w/g, (c) => c.toUpperCase());
	}

	function rawurl(r: Resource) {
		const b = r.branch || 'main';
		return `https://raw.githubusercontent.com/${r.owner}/${r.repo}/${b}/${encodeURI(r.file_path)}`;
	}

	function handlekeydown(e: KeyboardEvent) {
		const activetag = document.activeElement?.tagName;
		if (e.key === '/' && activetag !== 'INPUT' && activetag !== 'TEXTAREA') {
			e.preventDefault();
			searchinput?.focus();
		}
	}

	async function loadcourse() {
		startloading();
		try {
			const data = await getcourse(slug);
			if (data) {
				course = data.course;
				resources = data.resources;
			}
		} finally {
			stoploading();
		}
	}

	function startloading() {
		loading = true;
		showloadingindicator = false;
		if (loadingtimer) clearTimeout(loadingtimer);
		loadingtimer = setTimeout(() => {
			if (loading) showloadingindicator = true;
		}, LOADING_DELAY_MS);
	}

	function stoploading() {
		loading = false;
		showloadingindicator = false;
		if (loadingtimer) {
			clearTimeout(loadingtimer);
			loadingtimer = null;
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

	async function downloadall() {
		if (downloading || resources.length === 0) return;
		downloading = true;
		try {
			const zip = new JSZip();
			const results = await Promise.allSettled(
				resources.map(async (r) => {
					const res = await fetch(rawurl(r));
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
		loadcourse();
	});

	onDestroy(() => {
		if (loadingtimer) clearTimeout(loadingtimer);
		if (debounce) clearTimeout(debounce);
	});
</script>

<svelte:window onkeydown={handlekeydown} />

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
			<div class="mb-6 flex items-end justify-between">
				<div>
					{#if abbreviation}
						<span
							class="rounded bg-muted px-2 py-0.5 font-mono text-xs font-medium text-muted-foreground"
							>{abbreviation}</span
						>
					{/if}
					<h1 class="mt-1 text-2xl font-bold tracking-tight">{titlecase(course.name)}</h1>
				</div>
				<Button
					variant="outline"
					onclick={downloadall}
					disabled={downloading || resources.length === 0}
				>
					{#if downloading}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						downloading...
					{:else}
						<Download class="mr-2 h-4 w-4" />
						download all
					{/if}
				</Button>
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
									onclick={() => window.open(rawurl(r), '_blank')}
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
