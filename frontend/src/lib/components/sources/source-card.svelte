<script lang="ts">
    import { refreshSource, type Source } from '$lib/api';
    import { Button } from '$lib/components/ui/button';
    import * as Card from '$lib/components/ui/card';
    import { RefreshCw, Github, GitBranch, Archive, AlertCircle, Clock } from '@lucide/svelte';
    import { createEventDispatcher } from 'svelte';

    const { source } = $props<{ source: Source }>();
    let refreshing = $state(false);

    const dispatch = createEventDispatcher();

    async function handleRefresh() {
        refreshing = true;
        const success = await refreshSource(source.id);
        refreshing = false;
        if (success) {
            dispatch('refresh');
        }
    }

    function timeAgo(dateStr: string | null) {
        if (!dateStr) return 'Never synced';
        const date = new Date(dateStr);
        const now = new Date();
        const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);
        
        let interval = seconds / 31536000;
        if (interval > 1) return Math.floor(interval) + ' years ago';
        interval = seconds / 2592000;
        if (interval > 1) return Math.floor(interval) + ' months ago';
        interval = seconds / 86400;
        if (interval > 1) return Math.floor(interval) + ' days ago';
        interval = seconds / 3600;
        if (interval > 1) return Math.floor(interval) + ' hours ago';
        interval = seconds / 60;
        if (interval > 1) return Math.floor(interval) + ' minutes ago';
        
        return 'Just now';
    }
    
    const statusColor = $derived.by(() => {
        switch (source.source_status) {
            case 'active': return 'text-success';
            case 'error': return 'text-destructive';
            case 'archived': return 'text-warning';
            case 'pending': return 'text-info';
            default: return 'text-muted-foreground';
        }
    });

    const hasError = $derived(source.source_status === 'error');
    const isArchived = $derived(source.source_status === 'archived');
</script>

<Card.Root class="group relative flex w-full flex-col justify-between gap-3 rounded-xl border border-border/60 bg-card/40 px-4 py-3 md:px-5 md:py-4 transition-colors hover:border-border">
    <div class="flex items-start justify-between gap-3">
        <div class="space-y-1 overflow-hidden">
            <div class="flex items-center gap-2 text-xs text-muted-foreground">
                <Github class="h-4 w-4 shrink-0 text-muted-foreground/80" />
                <span class="truncate font-mono">{source.owner}</span>
            </div>
            <Card.Title class="truncate text-sm font-medium tracking-tight">
                {source.repo}
            </Card.Title>
            <div class="flex items-center gap-2 text-xs text-muted-foreground">
                <GitBranch class="h-3 w-3" />
                <span class="truncate">{source.branch}</span>
            </div>
        </div>

        <div class="flex flex-col items-end gap-2 text-right">
            <div class={`flex items-center gap-1 rounded-full border px-2 py-0.5 text-[10px] font-medium uppercase tracking-wide ${statusColor} border-border/60 bg-background/60`}>
                <span class="h-1.5 w-1.5 rounded-full bg-current"></span>
                {#if hasError}
                    <AlertCircle class="h-3 w-3" />
                {:else if isArchived}
                    <Archive class="h-3 w-3" />
                {/if}
                <span class="truncate">{source.source_status}</span>
            </div>
            <div class="flex items-center gap-1 text-[11px] text-muted-foreground">
                <Clock class="h-3 w-3" />
                <span>{timeAgo(source.last_synced_at)}</span>
            </div>
        </div>
    </div>

    <div class="flex items-center justify-end gap-2 pt-1">
        <Button
            variant="outline"
            size="sm"
            onclick={handleRefresh}
            disabled={refreshing}
            class="h-7 px-3 text-xs text-muted-foreground border-border/60 hover:bg-background/70"
        >
            <RefreshCw class={`mr-1.5 h-3.5 w-3.5 ${refreshing ? 'animate-spin' : ''}`} />
            <span>Sync</span>
        </Button>
    </div>
</Card.Root>
