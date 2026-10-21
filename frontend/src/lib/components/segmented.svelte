<script lang="ts">
	import type { Component } from 'svelte';
	import * as ToggleGroup from '$lib/components/ui/toggle-group';

	type Option = { value: string; label: string; icon?: Component };

	let {
		value = $bindable(),
		options,
		label
	}: { value: string; options: Option[]; label: string } = $props();

	let index = $derived(options.findIndex((o) => o.value === value));
</script>

<ToggleGroup.Root
	type="single"
	{value}
	onValueChange={(v) => v && (value = v)}
	aria-label={label}
	class="relative grid h-8 auto-cols-fr grid-flow-col rounded-lg bg-muted p-0.5"
>
	<span
		class="absolute inset-y-0.5 left-0.5 rounded-md bg-background shadow-sm transition-transform duration-200 ease-out"
		style="width: calc((100% - 4px) / {options.length}); transform: translateX({index * 100}%)"
	></span>
	{#each options as o (o.value)}
		<ToggleGroup.Item
			value={o.value}
			aria-label={o.label}
			class="relative h-7 rounded-md bg-transparent px-3 text-[13px] font-normal text-muted-foreground transition-colors group-data-[spacing=0]/toggle-group:rounded-md hover:bg-transparent data-[state=on]:bg-transparent data-[state=on]:text-foreground"
		>
			{#if o.icon}<o.icon class="size-3.5" />{:else}{o.label}{/if}
		</ToggleGroup.Item>
	{/each}
</ToggleGroup.Root>
