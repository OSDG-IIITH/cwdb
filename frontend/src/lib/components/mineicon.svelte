<script lang="ts">
	import { Minus, Pin, Plus } from '@lucide/svelte';
	import { user } from '$lib/auth';

	let { added, filled = '' }: { added: boolean; filled?: string } = $props();
</script>

{#if !$user}
	<Pin class={added ? filled : ''} />
{:else if added}
	<Minus />
{:else}
	<Plus />
{/if}
