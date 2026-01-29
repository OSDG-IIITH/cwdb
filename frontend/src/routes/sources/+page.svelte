<script lang="ts">
    import { listSources, type Source } from '$lib/api';
    import { onMount } from 'svelte';
    import SourceCard from '$lib/components/sources/source-card.svelte';
    import NewSourceDialog from '$lib/components/sources/new-source-dialog.svelte';
    import { Loader2 } from '@lucide/svelte';

    let sources = $state<Source[]>([]);
    let loading = $state(true);

    async function loadSources() {
        loading = true;
        sources = await listSources();
        loading = false;
    }

    onMount(() => {
        loadSources();
    });
</script>

<div class="min-h-screen bg-background text-foreground flex flex-col font-sans">
    <header class="border-b border-border/40 px-6 py-4 flex items-center justify-between sticky top-0 bg-background/95 backdrop-blur z-10 support-[backdrop-filter]:bg-background/60">
        <div class="flex items-center gap-2 font-bold text-xl tracking-tight font-mono">
            cwdb
        </div>
    </header>

    <main class="flex-1 container mx-auto p-6 max-w-6xl">
        <div class="flex items-center justify-between mb-8">
            <div class="space-y-1">
                 <h1 class="text-3xl font-bold tracking-tight">Sources</h1>
                 <p class="text-muted-foreground text-sm">Manage indexed GitHub repositories.</p>
            </div>
            <NewSourceDialog on:created={loadSources} />
        </div>

        {#if loading}
             <div class="flex justify-center items-center py-20">
                <Loader2 class="w-8 h-8 animate-spin text-muted-foreground" />
            </div>
        {:else if sources.length === 0}
            <div class="text-center py-20 border rounded-xl bg-muted/20 border-dashed">
                <p class="text-muted-foreground mb-4">No sources found.</p>
            </div>
        {:else}
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                {#each sources as source (source.id)}
                    <SourceCard {source} on:refresh={loadSources} />
                {/each}
            </div>
        {/if}
    </main>
</div>
