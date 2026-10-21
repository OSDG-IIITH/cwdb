<script lang="ts">
	import { FolderTree } from '@lucide/svelte';
	import { allresources } from '$lib/sync';
	import { sourcelist } from '$lib/courseview';

	let count = $derived(sourcelist($allresources).length);
</script>

<div class="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center">
	<div class="flex size-10 items-center justify-center rounded-xl border bg-card shadow-xs">
		<FolderTree class="size-4.5 text-muted-foreground" />
	</div>
	<div class="text-sm font-medium">Pick a source</div>
	<p class="max-w-xs text-[13px] text-muted-foreground">
		Browse {count} GitHub repos as folders, with every file reachable even when it has no course.
	</p>
</div>
