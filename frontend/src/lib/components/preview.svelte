<script lang="ts">
	import { ChevronLeft, ChevronRight, Download, ExternalLink, Pencil, X } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import Segmented from '$lib/components/segmented.svelte';
	import Viewer from '$lib/components/viewer.svelte';
	import { githuburl } from '$lib/utils';
	import { downloadfile } from '$lib/zip';
	import type { Resource } from '$lib/sync';

	let {
		r,
		pair = null,
		mobile = false,
		admin = false,
		onclose,
		onpick,
		onedit,
		onprev,
		onnext,
		ongosrc
	}: {
		r: Resource;
		pair?: Resource | null;
		mobile?: boolean;
		admin?: boolean;
		onclose: () => void;
		onpick: (id: string) => void;
		onedit: () => void;
		onprev: () => void;
		onnext: () => void;
		ongosrc: () => void;
	} = $props();

	let size = $derived(
		r.size
			? r.size > 1048576
				? `${(r.size / 1048576).toFixed(1)} MB`
				: `${Math.round(r.size / 1024)} KB`
			: ''
	);
	let paper = $derived(r.key ? (pair ?? r) : r);
	let keyed = $derived(r.key ? r : pair);
	let side = $derived(r.key ? 'key' : 'paper');
</script>

<div class="flex min-h-0 flex-1 flex-col">
	<div class="flex shrink-0 items-center gap-2 border-b bg-background py-2 pr-2 pl-4">
		<div class="flex min-w-0 flex-1 flex-col gap-px">
			<span class="truncate text-sm font-semibold">{r.title}</span>
			<span class="truncate text-[11px] text-muted-foreground">
				<button type="button" class="hover:underline" onclick={ongosrc}>{r.owner}/{r.repo}</button>
				<span class="font-mono">/{r.file_path}{size ? ` · ${size}` : ''}</span>
			</span>
		</div>
		{#if pair && keyed}
			<Segmented
				label="Paper or key"
				bind:value={() => side, (v) => onpick(v === 'key' ? keyed.id : paper.id)}
				options={[
					{ value: 'paper', label: 'Paper' },
					{ value: 'key', label: 'Key' }
				]}
			/>
		{/if}
		{#if admin}
			<Button variant="ghost" size="icon" aria-label="Edit metadata" onclick={onedit}>
				<Pencil />
			</Button>
		{/if}
		{#if !mobile}
			<Button
				variant="ghost"
				size="icon"
				aria-label="Download file"
				onclick={() => downloadfile(r)}
			>
				<Download />
			</Button>
			<Button
				variant="ghost"
				size="icon"
				aria-label="Open on GitHub"
				href={githuburl(r)}
				target="_blank"
				rel="noreferrer"
			>
				<ExternalLink />
			</Button>
		{/if}
		<Button variant="ghost" size="icon" aria-label="Close preview" onclick={onclose}><X /></Button>
	</div>
	{#key r.id}
		<Viewer {r} />
	{/key}
	{#if mobile}
		<div class="flex shrink-0 items-center gap-2 px-4 pt-2 pb-8">
			<Button
				variant="outline"
				size="icon"
				class="size-12"
				aria-label="Previous file"
				onclick={onprev}><ChevronLeft /></Button
			>
			<Button class="h-12 flex-1" onclick={() => downloadfile(r)}><Download />Download</Button>
			<Button
				variant="outline"
				size="icon"
				class="size-12"
				aria-label="Open on GitHub"
				href={githuburl(r)}
				target="_blank"
				rel="noreferrer"
			>
				<ExternalLink />
			</Button>
			<Button variant="outline" size="icon" class="size-12" aria-label="Next file" onclick={onnext}
				><ChevronRight /></Button
			>
		</div>
	{/if}
</div>
