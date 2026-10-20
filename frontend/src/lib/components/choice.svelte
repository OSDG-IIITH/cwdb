<script lang="ts">
	import * as Select from '$lib/components/ui/select';
	import { Label } from '$lib/components/ui/label';

	type Option = { value: string; label: string };

	let {
		value = $bindable(),
		options,
		label
	}: { value: string; options: Option[]; label: string } = $props();
</script>

<div class="flex flex-col gap-1.5">
	<Label>{label}</Label>
	<Select.Root type="single" bind:value>
		<Select.Trigger class="w-full">
			{options.find((o) => o.value === value)?.label ?? 'None'}
		</Select.Trigger>
		<Select.Content>
			<Select.Item value="" label="None">None</Select.Item>
			{#each options as o (o.value)}
				<Select.Item value={o.value} label={o.label}>{o.label}</Select.Item>
			{/each}
		</Select.Content>
	</Select.Root>
</div>
