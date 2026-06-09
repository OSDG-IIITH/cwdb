<script lang="ts">
	import { syncstatus } from '$lib/sync';
	import { Loader2, WifiOff, CheckCircle2, RefreshCw } from '@lucide/svelte';

	let status = $derived($syncstatus);

	function fmtdate(d: Date | null): string {
		if (!d) return 'never';
		const diff = Date.now() - new Date(d).getTime();
		if (diff < 60000) return 'just now';
		if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
		if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
		return new Date(d).toLocaleDateString();
	}
</script>

<div class="flex items-center gap-1.5 text-xs text-muted-foreground">
	{#if status.syncing}
		<Loader2 class="h-3 w-3 animate-spin" />
		<span>syncing...</span>
	{:else if !status.online && status.empty}
		<WifiOff class="h-3 w-3" />
		<span>offline — connect to VPN to load data</span>
	{:else if !status.online}
		<WifiOff class="h-3 w-3" />
		<span>offline</span>
	{:else}
		<span>synced {fmtdate(status.synced_at)}</span>
	{/if}
</div>
