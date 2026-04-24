<script lang="ts">
	import ResourceExplorer from '$lib/components/resources/resource-explorer.svelte';
	import { listAllResources, listSources, type Resource, type Source } from '$lib/api';
	import { Loader2 } from '@lucide/svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import { page } from '$app/state';
	import { onDestroy } from 'svelte';
	import { makeloadingstate } from '$lib/hooks/loading';

	let resources = $state<Resource[]>([]);
	let sources = $state<Source[]>([]);
	let loading = $state(true);
	let showLoadingIndicator = $state(false);
	let loadingcontrol = makeloadingstate(
		(value) => {
			loading = value;
		},
		(value) => {
			showLoadingIndicator = value;
		}
	);

	let path = $derived(page.params.path ? page.params.path.split('/') : []);

	let currentOwner = $derived(path.length >= 2 ? path[0] : null);
	let currentRepo = $derived(path.length >= 2 ? path[1] : null);

	let lastFetched: string | null = null;

	$effect(() => {
		const repoKey = currentOwner && currentRepo ? `${currentOwner}/${currentRepo}` : '';
		if (repoKey !== lastFetched) {
			lastFetched = repoKey;
			loadData(currentOwner, currentRepo);
		}
	});

	async function loadData(owner: string | null, repo: string | null) {
		loadingcontrol.start();
		try {
			if (owner && repo) {
				resources = await listAllResources(owner, repo);
			} else {
				sources = await listSources();
			}
		} finally {
			loadingcontrol.stop();
		}
	}

	onDestroy(() => {
		loadingcontrol.destroy();
	});
</script>

<div class="flex min-h-screen flex-col bg-background font-sans text-foreground">
	<PageHeader title="Resources" />

	<main class="container mx-auto max-w-6xl flex-1 p-6">
		<div class="mb-8 flex flex-col justify-between gap-4 md:flex-row md:items-center"></div>

		{#if loading && showLoadingIndicator}
			<div class="flex items-center justify-center py-20">
				<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
			</div>
		{:else if loading}
			<div class="py-20" aria-hidden="true"></div>
		{:else}
			<ResourceExplorer {resources} {sources} {path} />
		{/if}
	</main>
</div>
