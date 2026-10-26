<script lang="ts">
	import { Check, Minus, Pin, Plus } from '@lucide/svelte';
	import { user } from '$lib/auth';

	let {
		added,
		filled = '',
		remove = false
	}: { added: boolean; filled?: string; remove?: boolean } = $props();
</script>

{#if !$user}
	<Pin class={added ? filled : ''} />
{:else if added && remove}
	<Minus />
{:else if added}
	<Check class="text-foreground" />
{:else}
	<Plus />
{/if}
