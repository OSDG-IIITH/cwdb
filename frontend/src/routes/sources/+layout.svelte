<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { page } from '$app/state';
	import { base } from '$app/paths';
	import { loadsources, sourceinfo } from '$lib/api';
	import { user } from '$lib/auth';
	import { sourcelist } from '$lib/courseview';
	import { allresources } from '$lib/sync';
	import { ago, cn } from '$lib/utils';
	import NewSourceDialog from '$lib/components/sources/new-source-dialog.svelte';

	let { children }: { children: Snippet } = $props();

	let admin = $derived($user?.role === 'admin');
	let review = $derived(page.route.id === '/sources/review');
	let index = $derived(!page.params.owner && !review);
	let synced = $derived(
		new Map($sourceinfo.map((s) => [`${s.owner}/${s.repo}`, s.last_synced_at]))
	);
	let rows = $derived.by(() => {
		const found = sourcelist($allresources);
		const empty = admin
			? $sourceinfo
					.filter((s) => !found.some((f) => f.owner === s.owner && f.repo === s.repo))
					.map((s) => ({ id: String(s.id), owner: s.owner, repo: s.repo, count: 0, courses: 0 }))
			: [];
		return [...found, ...empty];
	});

	onMount(loadsources);
</script>

<div class="flex h-[calc(100dvh-3rem)] overflow-hidden bg-background text-foreground">
	<aside
		class={cn(
			'w-full shrink-0 flex-col gap-px overflow-y-auto border-r px-2 py-3 text-[13px] lg:w-80',
			index ? 'flex' : review ? 'hidden' : 'hidden lg:flex'
		)}
	>
		<div class="flex justify-between px-2.5 pb-2.5 text-xs text-muted-foreground">
			<span>{rows.length} sources</span>
			<span>files</span>
		</div>
		{#each rows as s (s.id)}
			{@const last = synced.get(`${s.owner}/${s.repo}`)}
			<a
				href="{base}/sources/{s.owner}/{s.repo}"
				class={cn(
					'flex h-11 flex-col justify-center rounded-md px-2.5 hover:bg-accent/50',
					page.params.owner === s.owner && page.params.repo === s.repo && 'bg-accent'
				)}
			>
				<span class="flex justify-between gap-3">
					<span class="truncate font-medium">{s.owner}/{s.repo}</span>
					<span class="font-mono text-xs text-muted-foreground">{s.count.toLocaleString()}</span>
				</span>
				<span class="text-[11px] text-muted-foreground">
					{s.courses} courses{last ? ` · synced ${ago(last)}` : ''}
				</span>
			</a>
		{/each}
		{#if admin}
			<div class="pt-3">
				<NewSourceDialog on:created={loadsources} />
			</div>
		{/if}
	</aside>

	<div class={cn('min-w-0 flex-1', index ? 'hidden lg:flex' : 'flex')}>
		{@render children()}
	</div>
</div>
