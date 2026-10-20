<script lang="ts">
	import ResourceExplorer from '$lib/components/resources/resource-explorer.svelte';
	import {
		search,
		getcourse,
		allresources,
		courses,
		mycourses,
		togglepin,
		kindtag,
		type Resource,
		type CourseInfo
	} from '$lib/sync';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { Loader2, Search, Download, Pin } from '@lucide/svelte';
	import { page } from '$app/state';
	import { base } from '$app/paths';
	import { onDestroy } from 'svelte';
	import { toast } from 'svelte-sonner';
	import JSZip from 'jszip';
	import { rawgithuburl } from '$lib/utils';

	let searchresults = $state<Resource[]>([]);
	let downloading = $state(false);
	let searchquery = $state('');
	let searching = $state(false);

	let debounce: ReturnType<typeof setTimeout> | null = null;

	let slug = $derived(page.params.slug ?? '');
	let path = $derived(page.params.path ? page.params.path.split('/') : []);
	let basepath = $derived(`${base}/courses/${slug}`);

	let course = $derived<CourseInfo | null>(
		slug === 'unclassified'
			? {
					code: '',
					name: 'Unclassified',
					shortnames: [],
					alias: '',
					semester: '',
					count: 0,
					kinds: []
				}
			: getcourse($courses, slug)
	);
	let resources = $derived(
		course
			? $allresources.filter((r) => (course.code ? r.course_id === course.code : !r.course_id))
			: []
	);
	let pinned = $derived(!!course?.code && $mycourses.includes(course.code));

	function capseason(s: string): string {
		return s.replace(/\b(monsoon|spring|both)\b/gi, (w) => w.charAt(0).toUpperCase() + w.slice(1));
	}

	function handlesearch() {
		if (debounce) clearTimeout(debounce);
		debounce = setTimeout(() => {
			if (!searchquery.trim()) {
				searching = false;
				searchresults = [];
				return;
			}
			searching = true;
			searchresults = search(searchquery, course?.code || null)
				.filter((e) => e.t === 'f' && (course?.code || !e.course))
				.map((e) => e.resource!);
		}, 200);
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
			if (failed > 0) toast.warning(`${failed} file(s) could not be downloaded`);
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

	onDestroy(() => {
		if (debounce) clearTimeout(debounce);
	});
</script>

<div class="flex flex-1 flex-col bg-background text-foreground">
	<main class="container mx-auto max-w-6xl flex-1 p-6">
		{#if course}
			<div class="relative mb-8">
				<Button
					variant="outline"
					size="lg"
					class="group/pin absolute top-0 right-0"
					onclick={() => togglepin(course!.code)}
					disabled={!course.code}
				>
					<Pin
						class={`h-4 w-4 transition-colors ${pinned ? 'fill-foreground' : 'group-hover/pin:fill-foreground/25'}`}
					/>
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
					{#if course.alias}
						<span
							class="rounded-md bg-muted/60 px-2 py-0.5 font-mono text-xs font-medium text-muted-foreground"
						>
							{course.alias}
						</span>
					{/if}
				</div>

				<h1 class="mt-3 text-4xl leading-tight font-extrabold tracking-tight md:text-5xl">
					{course.name}
				</h1>

				<div class="mt-6 flex flex-wrap items-end justify-between gap-4">
					<div class="flex flex-wrap items-start gap-x-8 gap-y-3">
						{#if course.semester}
							<div>
								<div
									class="mb-1 text-[10px] font-semibold tracking-widest text-muted-foreground/60 uppercase"
								>
									Season
								</div>
								<div class="text-sm text-foreground">
									{capseason(course.semester)}
								</div>
							</div>
						{/if}
						<div>
							<div
								class="mb-1 text-[10px] font-semibold tracking-widest text-muted-foreground/60 uppercase"
							>
								Resources
							</div>
							<div class="text-sm text-foreground">{resources.length}</div>
						</div>
					</div>

					<Button size="lg" onclick={downloadall} disabled={downloading || resources.length === 0}>
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
					data-slash
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
									<div class="text-base font-medium break-words text-foreground">{r.title}</div>
									<div class="mt-1 text-xs break-all text-muted-foreground">{r.file_path}</div>
									<div class="mt-3 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
										<span class="rounded bg-muted px-2 py-0.5 break-all">{r.owner}/{r.repo}</span>
										<span class="rounded bg-muted px-2 py-0.5">{kindtag(r)}</span>
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
