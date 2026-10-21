<script lang="ts">
	import { goto } from '$app/navigation';
	import { ArrowRight, FileText, Folder, Github, Pin, Search, X } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Kbd } from '$lib/components/ui/kbd';
	import Segmented from '$lib/components/segmented.svelte';
	import Preview from '$lib/components/preview.svelte';
	import { courses, mycourses, search, togglepin, type Entry } from '$lib/sync';
	import { entryhref, entrysub, entrytag } from '$lib/entries';
	import { cn } from '$lib/utils';

	let { query, onquery }: { query: string; onquery: (q: string) => void } = $props();

	const limits = { c: 12, d: 20, f: 60 };
	const names = { c: 'Courses', d: 'Folders', f: 'Files' };

	let only = $state('all');
	let picked = $state<string | null>(null);

	let info = $derived(new Map($courses.map((c) => [c.code, c])));
	let hits = $derived.by(() => {
		const tier = (e: Entry) => (e.course && $mycourses.includes(e.course) ? 0 : 1);
		return $courses.length ? search(query, null).sort((a, b) => tier(a) - tier(b)) : [];
	});
	let all = $derived(
		(['c', 'd', 'f'] as const).map((t) => ({
			t,
			rows: hits.filter((e) => e.t === t).slice(0, limits[t])
		}))
	);
	let shown = $derived(all.some((g) => g.t === only && g.rows.length) ? only : 'all');
	let groups = $derived(all.filter((g) => g.rows.length && (shown === 'all' || shown === g.t)));
	let flat = $derived(groups.flatMap((g) => g.rows));
	let sel = $derived(flat.find((e) => e.id === picked) ?? flat[0] ?? null);

	const alias = (e: Entry) => (e.course && info.get(e.course)?.alias) || '—';

	/** moves the selection by one row and keeps it in view */
	function move(by: number) {
		if (!flat.length) return;
		const i = Math.max(
			0,
			flat.findIndex((e) => e.id === sel?.id)
		);
		picked = flat[(i + by + flat.length) % flat.length].id;
		requestAnimationFrame(() =>
			document.querySelector(`[data-hit="${picked}"]`)?.scrollIntoView({ block: 'nearest' })
		);
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
			e.preventDefault();
			move(e.key === 'ArrowDown' ? 1 : -1);
		} else if (e.key === 'Enter' && sel) goto(entryhref(sel));
		else if (e.key === 'Escape') onquery('');
	}
</script>

{#snippet hit(e: Entry)}
	<button
		type="button"
		data-hit={e.id}
		class={cn(
			'flex min-h-14 w-full items-center gap-3 rounded-lg px-3 py-2 text-left transition-colors hover:bg-accent/60',
			sel?.id === e.id && 'bg-accent hover:bg-accent'
		)}
		onclick={() => (picked = e.id)}
		ondblclick={() => goto(entryhref(e))}
	>
		<span
			class="flex size-9 shrink-0 items-center justify-center rounded-lg border bg-card font-mono text-[10px] text-muted-foreground shadow-xs"
		>
			{#if e.t === 'c'}{alias(e).slice(0, 4)}{:else if e.t === 'd'}<Folder
					class="size-4"
				/>{:else}<FileText class="size-4" />{/if}
		</span>
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class="truncate text-sm font-medium">{e.title}</span>
			<span class="truncate text-xs text-muted-foreground"
				>{e.t === 'c' ? entrysub(e) : `${alias(e)} · ${entrysub(e)}`}</span
			>
		</span>
		{#if e.t !== 'c'}
			<span class="shrink-0 text-xs text-muted-foreground/70">{entrytag(e)}</span>
		{/if}
	</button>
{/snippet}

<div class="flex h-[calc(100dvh-3.75rem)] flex-col">
	<div class="shrink-0 border-b px-6 pt-2 pb-4">
		<div class="mx-auto flex max-w-7xl items-center gap-4">
			<label class="relative flex min-w-0 flex-1 items-center">
				<Search class="pointer-events-none absolute left-4 size-4.5 text-muted-foreground" />
				<input
					{@attach (el) => {
						el.focus();
						el.setSelectionRange(el.value.length, el.value.length);
					}}
					value={query}
					oninput={(e) => {
						picked = null;
						onquery(e.currentTarget.value);
					}}
					{onkeydown}
					placeholder="Search courses, papers, slides"
					aria-label="Search"
					autocomplete="off"
					spellcheck="false"
					class="h-12 w-full rounded-xl border bg-card pr-24 pl-11 text-base shadow-xs outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/20"
				/>
				<Button
					variant="ghost"
					size="sm"
					class="absolute right-2 h-8 gap-1.5 text-muted-foreground"
					onclick={() => onquery('')}
				>
					<X />Clear
				</Button>
			</label>
			<Segmented
				bind:value={() => shown, (v) => (only = v)}
				label="Show"
				options={[
					{ value: 'all', label: `All ${hits.length > 99 ? '99+' : hits.length}` },
					...all
						.filter((g) => g.rows.length)
						.map((g) => ({ value: g.t, label: `${names[g.t]} ${g.rows.length}` }))
				]}
			/>
		</div>
	</div>

	<div class="mx-auto flex min-h-0 w-full max-w-7xl flex-1">
		<div class="flex w-[46%] min-w-80 shrink-0 flex-col overflow-y-auto border-r px-3 py-3 lg:pl-6">
			{#each groups as g (g.t)}
				<div class="px-3 pt-3 pb-1.5 text-xs font-medium text-muted-foreground">
					{names[g.t]}
				</div>
				{#each g.rows as e (e.id)}{@render hit(e)}{/each}
			{:else}
				<div class="flex flex-1 flex-col items-center justify-center gap-2 text-center">
					<div class="text-sm font-medium">Nothing matches “{query}”</div>
					<p class="max-w-xs text-[13px] text-muted-foreground">
						Try a course alias like <span class="font-mono">dsa</span>, a kind like
						<span class="font-mono">mid</span>, or a year.
					</p>
				</div>
			{/each}
			{#if flat.length}
				<div
					class="mt-auto flex gap-4 px-3 pt-6 pb-1 text-[11px] whitespace-nowrap text-muted-foreground/80"
				>
					<span><Kbd>↑</Kbd> <Kbd>↓</Kbd> move</span>
					<span><Kbd>↵</Kbd> open</span>
					<span><Kbd>esc</Kbd> clear</span>
				</div>
			{/if}
		</div>

		<div class="flex min-w-0 flex-1 flex-col">
			{#if sel?.t === 'f' && sel.resource}
				<Preview
					r={sel.resource}
					onclose={() => onquery('')}
					onpick={() => {}}
					onedit={() => {}}
					onprev={() => move(-1)}
					onnext={() => move(1)}
					ongosrc={() => goto(entryhref(sel!))}
				/>
			{:else if sel}
				{@const c = sel.course ? info.get(sel.course) : null}
				<div class="flex flex-1 flex-col justify-center gap-6 px-12 py-10">
					<div class="flex flex-col gap-2">
						<div class="font-mono text-xs text-muted-foreground">
							{sel.t === 'c' ? `${c?.alias} · ${c?.code}` : `${alias(sel)} · folder`}
						</div>
						<div class="text-2xl leading-tight font-semibold tracking-tight">
							{sel.t === 'c' ? c?.name : sel.title}
						</div>
						<div class="text-sm break-all text-muted-foreground">
							{sel.t === 'c'
								? `${sel.count} files${c?.kinds.length ? ` · ${c.kinds.join(', ')}` : ''}`
								: `${sel.path} · ${sel.count} files`}
						</div>
					</div>
					<div class="flex flex-wrap gap-2">
						<Button href={entryhref(sel)}>
							{sel.t === 'c' ? 'Open course' : 'Open folder'}<ArrowRight />
						</Button>
						{#if sel.t === 'c' && sel.course}
							{@const code = sel.course}
							<Button variant="outline" onclick={() => togglepin(code)}>
								<Pin class={$mycourses.includes(code) ? 'fill-foreground' : ''} />
								{$mycourses.includes(code) ? 'In my courses' : 'Add to my courses'}
							</Button>
						{:else if sel.github}
							<Button variant="outline" href={sel.github} target="_blank" rel="noreferrer">
								<Github />GitHub
							</Button>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>
