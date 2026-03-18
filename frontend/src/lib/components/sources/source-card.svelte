<script lang="ts">
    import { refreshSource, toggleSourceLike, type Source } from '$lib/api';
    import { Button } from '$lib/components/ui/button';
    import * as Card from '$lib/components/ui/card';
    import { RefreshCw, Github, GitBranch, Archive, AlertCircle, Clock, Heart } from '@lucide/svelte';
    import { user } from '$lib/auth';
    import { toast } from 'svelte-sonner';

    const { source, onrefresh } = $props<{ source: Source; onrefresh?: () => void }>();
    let refreshing = $state(false);
    let localLikeState = $state<{ liked: boolean; like_count: number } | null>(null);
    let liked = $derived(localLikeState?.liked ?? source.liked);
    let likeCount = $derived(localLikeState?.like_count ?? source.like_count);

    async function handleRefresh() {
        if (!$user) {
            toast.error("Unable to sync source.", {
                description: "You must be logged in to sync a source."
            });
            return;
        }
        refreshing = true;
        const success = await refreshSource(source.id);
        refreshing = false;
        if (success) {
            toast.success("Source Synced", {
                description: `${source.owner}/${source.repo} has been successfully updated.`,
            });
            onrefresh?.();
        }
    }

    async function handleLike() {
        const result = await toggleSourceLike(source.id);
        if (result) {
            localLikeState = result;
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

        <div class="flex items-center gap-1">
             <span class="text-xs text-muted-foreground tabular-nums">{likeCount}</span>
             <Button variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground hover:text-foreground" onclick={handleLike}>
                 <Heart class={`h-4 w-4 transition-colors ${liked ? 'fill-destructive text-destructive' : ''}`} />
                 <span class="sr-only">Like</span>
            </Button>
        </div>
    </div>

    <div class="flex items-center justify-between gap-2 pt-1">
        <div class="flex items-center gap-1 text-[11px] text-muted-foreground">
            <Clock class="h-3 w-3" />
            <span>{timeAgo(source.last_synced_at)}</span>
        </div>
        {#if $user && $user.role === 'admin'}
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
        {/if}
    </div>
</Card.Root>
