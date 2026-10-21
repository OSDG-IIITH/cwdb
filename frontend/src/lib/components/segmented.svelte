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
	class="relative grid h-9 auto-cols-fr grid-flow-col rounded-xl bg-muted p-[3px]"
>
	<span
		class="absolute inset-y-[3px] left-[3px] rounded-lg border border-black/5 bg-background shadow-sm transition-transform duration-200 ease-out dark:border-white/10 dark:bg-accent"
		style="width: calc((100% - 6px) / {options.length}); transform: translateX({index * 100}%)"
	></span>
	{#each options as o (o.value)}
		<ToggleGroup.Item
			value={o.value}
			aria-label={o.label}
			class="relative h-full rounded-lg bg-transparent px-3 text-sm font-medium text-muted-foreground transition-colors group-data-[spacing=0]/toggle-group:rounded-lg hover:text-foreground hover:bg-transparent data-[state=on]:bg-transparent data-[state=on]:text-foreground"
		>
			{#if o.icon}<o.icon class="size-3.5" />{:else}{o.label}{/if}
		</ToggleGroup.Item>
	{/each}
</ToggleGroup.Root>
