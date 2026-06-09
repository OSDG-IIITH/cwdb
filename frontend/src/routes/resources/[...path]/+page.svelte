<script lang="ts">
	import ResourceExplorer from '$lib/components/resources/resource-explorer.svelte';
	import { allresources, type Resource } from '$lib/sync';
	import { Loader2 } from '@lucide/svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import { page } from '$app/state';
	import { syncstatus } from '$lib/sync';

	let path = $derived(page.params.path ? page.params.path.split('/') : []);
	let resources = $derived($allresources);
	let loading = $derived($syncstatus.syncing && resources.length === 0);
</script>

<div class="flex min-h-screen flex-col bg-background font-sans text-foreground">
	<PageHeader title="Resources" />

	<main class="container mx-auto max-w-6xl flex-1 p-6">
		{#if loading}
			<div class="flex items-center justify-center py-20">
				<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
			</div>
		{:else}
			<ResourceExplorer {resources} {path} />
		{/if}
	</main>
</div>
