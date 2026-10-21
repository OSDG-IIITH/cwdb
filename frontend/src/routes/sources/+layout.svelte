<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { page } from '$app/state';
	import { base } from '$app/paths';
	import { loadsources, sourceinfo } from '$lib/api';
	import { user } from '$lib/auth';
	import { sourcelist } from '$lib/courseview';
	import { allresources } from '$lib/sync';
	import { ago, cn } from '$lib/utils';
	import Resize from '$lib/components/resize.svelte';
	import NewSourceDialog from '$lib/components/sources/new-source-dialog.svelte';

	let { children }: { children: Snippet } = $props();

	let listwidth = $state(320);
	let admin = $derived($user?.role === 'admin');
	let review = $derived(page.route.id === '/sources/review');
	let index = $derived(!page.params.owner && !review);
	let open = $derived(page.url.searchParams.get('f'));
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
			'relative w-full shrink-0 flex-col border-r bg-sidebar lg:w-(--w)',
			index ? 'flex' : review ? 'hidden' : 'hidden lg:flex'
		)}
		style="--w: {listwidth}px"
	>
		<Resize
			bind:width={listwidth}
			min={240}
			max={480}
			fallback={320}
			side="right"
			key="sourcelist"
		/>
		<div class="flex min-h-0 flex-1 flex-col gap-px overflow-y-auto px-2 py-3 text-[13px]">
			<div class="flex justify-between px-2.5 pb-2 text-[11px] font-medium text-muted-foreground/70">
				<span>{rows.length} sources</span>
				<span>files</span>
			</div>
			{#each rows as s (s.id)}
				{@const last = synced.get(`${s.owner}/${s.repo}`)}
				<a
					href="{base}/sources/{s.owner}/{s.repo}{open ? `?f=${open}` : ''}"
					class={cn(
						'flex h-12 flex-col justify-center gap-0.5 rounded-md px-2.5 transition-colors hover:bg-accent/60',
						page.params.owner === s.owner && page.params.repo === s.repo && 'bg-accent'
					)}
				>
					<span class="flex justify-between gap-3">
						<span class="truncate font-medium">{s.repo}</span>
						<span class="text-xs text-muted-foreground">{s.count.toLocaleString()}</span>
					</span>
					<span class="truncate text-[11px] text-muted-foreground">
						{s.owner} · {s.courses} courses{last ? ` · ${ago(last)}` : ''}
					</span>
				</a>
			{/each}
			{#if admin}
				<div class="pt-3">
					<NewSourceDialog on:created={loadsources} />
				</div>
			{/if}
		</div>
	</aside>

	<div class={cn('min-w-0 flex-1', index ? 'hidden lg:flex' : 'flex')}>
		{@render children()}
	</div>
</div>
