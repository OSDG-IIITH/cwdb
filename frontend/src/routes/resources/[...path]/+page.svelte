<script lang="ts">
    import ResourceExplorer from '$lib/components/resources/resource-explorer.svelte';
    import { listAllResources, type Resource } from '$lib/api';
    import { onMount } from 'svelte';
    import { Loader2 } from '@lucide/svelte';
    import PageHeader from '$lib/components/page-header.svelte';
    import { page } from '$app/state';

    let resources = $state<Resource[]>([]);
    let loading = $state(true);

    async function loadResources() {
        loading = true;
        resources = await listAllResources();
        loading = false;
    }

    onMount(() => {
        loadResources();
    });

    let path = $derived(page.params.path ? page.params.path.split('/') : []);
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
            <ResourceExplorer {resources} {path} />
        {/if}
    </main>
</div>
