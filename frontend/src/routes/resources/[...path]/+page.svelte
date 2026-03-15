<script lang="ts">
    import ResourceExplorer from '$lib/components/resources/resource-explorer.svelte';
    import { listAllResources, listSources, type Resource, type Source } from '$lib/api';
    import { Loader2 } from '@lucide/svelte';
    import PageHeader from '$lib/components/page-header.svelte';
    import { page } from '$app/state';

    let resources = $state<Resource[]>([]);
    let sources = $state<Source[]>([]);
    let loading = $state(true);

    let path = $derived(page.params.path ? page.params.path.split('/') : []);
    
    let currentOwner = $derived(path.length >= 2 ? path[0] : null);
    let currentRepo = $derived(path.length >= 2 ? path[1] : null);

    let lastFetched = $state<string | null>(null);

    $effect(() => {
        const repoKey = currentOwner && currentRepo ? `${currentOwner}/${currentRepo}` : '';
        if (repoKey !== lastFetched) {
            lastFetched = repoKey;
            loadData(currentOwner, currentRepo);
        }
    });

    async function loadData(owner: string | null, repo: string | null) {
        loading = true;
        if (owner && repo) {
            resources = await listAllResources(owner, repo);
        } else {
            sources = await listSources();
        }
        loading = false;
    }
</script>

<div class="min-h-screen bg-background text-foreground flex flex-col font-sans">
    <PageHeader title="Resources" />

    <main class="flex-1 container mx-auto p-6 max-w-6xl">
        <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-8">
        </div>

        {#if loading}
            <div class="flex justify-center items-center py-20">
                <Loader2 class="w-8 h-8 animate-spin text-muted-foreground" />
            </div>
        {:else}
            <ResourceExplorer {resources} {sources} {path} />
        {/if}
    </main>
</div>
