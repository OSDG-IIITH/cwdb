<script lang="ts">
	import { ChevronRight, File, Folder } from '@lucide/svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { courses, kindtag, type Resource } from '$lib/sync';
	import type { Group, TreeRow } from '$lib/courseview';
	import { cn, coursehref } from '$lib/utils';

	let {
		groups,
		tree,
		active,
		picked,
		admin,
		showkind,
		courselinks = false,
		ontoggle,
		onopen,
		onfold
	}: {
		groups: Group[];
		tree: TreeRow[] | null;
		active: string | null;
		picked: Set<string>;
		admin: boolean;
		showkind: boolean;
		courselinks?: boolean;
		ontoggle: (r: Resource) => void;
		onopen: (r: Resource) => void;
		onfold: (id: string) => void;
	} = $props();

	let unfolded = new SvelteSet<string>();
	let aliases = $derived(new Map($courses.map((c) => [c.code, c.alias || c.code])));

	const fold = (id: string) => (unfolded.has(id) ? unfolded.delete(id) : unfolded.add(id));

	/** marks needing review or hidden, shown to admins only */
	const mark = (r: Resource) => (r.hidden ? 'hidden' : r.review ? 'review' : '');
</script>

{#snippet file(r: Resource, depth: number, tall: boolean, tag: string, more?: () => void, alts = 0)}
	<div
		class={cn(
			'group flex items-center gap-3 rounded-md px-3 transition-colors hover:bg-accent/60',
			tall ? 'h-[52px] border-b max-lg:rounded-none lg:h-[38px] lg:border-b-0' : 'h-[34px]',
			active === r.id && 'bg-accent',
			admin && r.hidden && 'opacity-50'
		)}
		data-row={r.id}
	>
		{#if alts}
			<button
				type="button"
				class="flex size-4 shrink-0 items-center justify-center text-muted-foreground hover:text-foreground"
				aria-label="Show sources"
				aria-expanded={unfolded.has(r.id)}
				onclick={more}
			>
				<ChevronRight
					class={cn('size-4 transition-transform', unfolded.has(r.id) && 'rotate-90')}
				/>
			</button>
		{:else}
			<Checkbox
				class={cn(
					'hidden transition-opacity group-hover:opacity-100 lg:flex',
					!picked.size && 'lg:opacity-0'
				)}
				checked={picked.has(r.id)}
				onCheckedChange={() => ontoggle(r)}
				aria-label="Select file"
			/>
		{/if}
		<button
			type="button"
			class="flex h-full min-w-0 flex-1 items-center gap-3 text-left"
			style="padding-left: {depth * 18}px"
			onclick={() => onopen(r)}
		>
			{#if !tall}<File class="size-3.5 shrink-0 text-muted-foreground/70" />{/if}
			<span class={cn('min-w-0 flex-1 truncate', tall ? 'text-[15px] lg:text-sm' : 'text-[13px]')}>
				{tall ? r.title : r.name}
			</span>
			{#if admin && mark(r)}
				<span class="rounded border px-1 font-mono text-[10px] text-muted-foreground"
					>{mark(r)}</span
				>
			{/if}
			{#if tag}
				<span
					class={cn(
						'shrink-0 text-[11px] text-muted-foreground',
						tall && 'rounded-md bg-muted px-1.5 py-0.5'
					)}>{tag}</span
				>
			{/if}
			{#if alts}
				<span class="hidden w-16 shrink-0 text-right text-xs text-muted-foreground/70 lg:block"
					>{alts + 1} sources</span
				>
			{/if}
		</button>
		{#if more && !alts}
			<button
				type="button"
				class="text-xs text-muted-foreground hover:text-foreground"
				onclick={more}
			>
				{unfolded.has(r.id) ? 'hide' : 'files'}
			</button>
		{/if}
	</div>
{/snippet}

<div class="flex-1 overflow-y-auto px-3 pb-24" data-list>
	{#if tree}
		<div class="pt-2">
			{#each tree as t (t.id)}
				{#if t.dir}
					<div
						class="flex h-[34px] items-center gap-2 rounded-md px-3 text-[13px] transition-colors hover:bg-accent/60"
						style="padding-left: {t.depth * 18 + 12}px"
					>
						<button
							type="button"
							class="flex h-full min-w-0 flex-1 items-center gap-2 text-left"
							onclick={() => onfold(t.id)}
						>
							<ChevronRight
								class={cn(
									'size-3 text-muted-foreground transition-transform',
									t.open && 'rotate-90'
								)}
							/>
							<Folder class="size-3.5 text-muted-foreground/70" />
							<span class="flex-1 truncate font-medium">{t.name}</span>
						</button>
						{#if courselinks && t.course}
							<a
								href={coursehref(t.course)}
								class="rounded-md border px-1.5 py-0.5 text-[11px] font-medium text-muted-foreground transition-colors hover:border-foreground/30 hover:text-foreground"
								>{aliases.get(t.course) ?? t.course}</a
							>
						{:else if courselinks && t.course === null}
							<span class="text-xs text-muted-foreground/70">unclassified</span>
						{/if}
						<span class="w-8 text-right text-[11px] text-muted-foreground/70">{t.count}</span>
					</div>
				{:else if t.r}
					{@render file(t.r, t.depth + 1, false, kindtag(t.r))}
				{/if}
			{/each}
		</div>
	{:else}
		{#each groups as g (g.name)}
			<div class="pt-2">
				<div class="sticky top-0 z-10 flex items-baseline gap-2 bg-background px-3 pt-2 pb-1.5">
					<span class="text-xs font-medium">{g.name}</span>
					<span class="text-[11px] text-muted-foreground/70">{g.rows.length}</span>
					{#if g.note}<span class="truncate text-xs text-muted-foreground">{g.note}</span>{/if}
				</div>
				{#each g.rows as row (row.r.id)}
					{@render file(
						row.r,
						0,
						true,
						showkind ? kindtag(row.r) : row.r.key ? 'key' : '',
						row.kids.length || row.alts.length ? () => fold(row.r.id) : undefined,
						row.alts.length
					)}
					{#if unfolded.has(row.r.id)}
						{#each row.alts.length ? [row.r, ...row.alts] : [] as alt (alt.id)}
							{@render file(alt, 1, false, alt.repo)}
						{/each}
						{#each row.kids as kid (kid.id)}
							{@render file(kid, 1, false, '')}
						{/each}
					{/if}
				{/each}
			</div>
		{:else}
			<div class="py-16 text-center text-sm text-muted-foreground">no files</div>
		{/each}
	{/if}
</div>
