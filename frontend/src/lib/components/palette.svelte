<script module lang="ts">
	import { writable, get } from 'svelte/store';
	import { page } from '$app/state';
	import { courses, getcourse } from '$lib/sync';

	export const palette = writable({ open: false, query: '', scope: null as string | null });

	/** opens search with an optional query, scoped to the given or current course */
	export function openpalette(query = '', scope?: string) {
		palette.set({
			open: true,
			query,
			scope: scope ?? getcourse(get(courses), page.params.slug)?.code ?? null
		});
	}

	/** closes search */
	export function closepalette() {
		palette.set({ open: false, query: '', scope: null });
	}
</script>

<script lang="ts">
	import { goto, pushState } from '$app/navigation';
	import { MediaQuery } from 'svelte/reactivity';
	import { Command as CommandPrimitive } from 'bits-ui';
	import { FileText, Folder, Search, X } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Command from '$lib/components/ui/command';
	import { Kbd } from '$lib/components/ui/kbd';
	import { search, getentry, mycourses, togglepin, type Entry } from '$lib/sync';
	import { entryhref, entrysub, entrytag } from '$lib/entries';

	const detail = false;
	const tries = ['dsa mid', 'osn endsem', 'la 2023'];
	const phone = new MediaQuery('max-width: 639px');

	let pushed = false;

	let active = $state('');
	let input = $state<HTMLInputElement | null>(null);

	let aliases = $derived(new Map($courses.map((c) => [c.code, c.alias])));
	let query = $derived($palette.query);
	let scope = $derived($palette.scope);
	let here = $derived(getcourse($courses, page.params.slug)?.code ?? null);

	let groups = $derived.by(() => {
		const tier = (e: Entry) =>
			e.course === here && here ? 0 : e.course && $mycourses.includes(e.course) ? 1 : 2;
		const hits = query.trim()
			? search(query, scope).sort((a, b) => tier(a) - tier(b))
			: scope
				? []
				: ($mycourses.map((c) => getentry(`c:${c}`)).filter(Boolean) as Entry[]);
		return [
			{ name: 'Courses', rows: scope ? [] : hits.filter((e) => e.t === 'c').slice(0, 5) },
			{ name: 'Folders', rows: hits.filter((e) => e.t === 'd').slice(0, 5) },
			{ name: 'Files', rows: hits.filter((e) => e.t === 'f').slice(0, 30) }
		].filter((g) => g.rows.length);
	});

	let sel = $derived.by(() => {
		const rows = groups.flatMap((g) => g.rows);
		return rows.find((e) => e.id === active) ?? rows[0] ?? null;
	});

	function alias(e: Entry): string {
		return (e.course && aliases.get(e.course)) || '—';
	}

	const sub = entrysub;
	const tag = entrytag;
	const href = entryhref;

	let full = $derived(phone.current && $palette.open);

	$effect(() => {
		if (full && !pushed) {
			pushState('', { search: true });
			pushed = true;
		}
	});

	$effect(() => {
		if (!page.state.search && pushed) {
			pushed = false;
			closepalette();
		}
	});

	$effect(() => {
		document.documentElement.classList.toggle('overflow-hidden', full);
	});

	/** closes the phone search through history so the back button and cancel agree */
	function dismiss() {
		if (pushed) history.back();
		else closepalette();
	}

	function primary(e: Entry) {
		const was = pushed;
		pushed = false;
		closepalette();
		goto(href(e), { replaceState: was });
	}

	function secondary(e: Entry) {
		if (e.t === 'c') return togglepin(e.course!);
		window.open(e.github, '_blank', 'noopener');
		closepalette();
	}

	async function copy(e: Entry) {
		await navigator.clipboard.writeText(location.origin + href(e));
		toast.success('Link copied');
	}

	function onkeydown(e: KeyboardEvent) {
		const mod = e.ctrlKey || e.metaKey;
		if (mod && e.key === 'Enter' && sel) {
			e.preventDefault();
			e.stopPropagation();
			secondary(sel);
		} else if (
			mod &&
			e.key.toLowerCase() === 'c' &&
			sel &&
			input?.selectionStart === input?.selectionEnd
		) {
			e.preventDefault();
			copy(sel);
		} else if (e.key === 'Backspace' && !query && scope) {
			e.preventDefault();
			palette.update((p) => ({ ...p, scope: null }));
		}
	}
</script>

{#snippet row(e: Entry)}
	<button
		type="button"
		class="flex min-h-15 w-full items-center gap-3 px-4 py-2 text-left active:bg-accent"
		onclick={() => primary(e)}
	>
		<span
			class="flex size-9 shrink-0 items-center justify-center rounded-lg border bg-card font-mono text-[10px] text-muted-foreground"
		>
			{#if e.t === 'c'}{alias(e).slice(0, 4)}{:else if e.t === 'd'}<Folder
					class="size-4"
				/>{:else}<FileText class="size-4" />{/if}
		</span>
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class="truncate text-[15px]">{e.title}</span>
			<span class="truncate text-xs text-muted-foreground"
				>{e.t === 'c' ? sub(e) : `${alias(e)} · ${sub(e)}`}</span
			>
		</span>
		{#if e.t !== 'c'}
			<span class="shrink-0 text-xs text-muted-foreground/70">{tag(e)}</span>
		{/if}
	</button>
{/snippet}

{#snippet fullscreen()}
	<div
		class="fixed inset-0 z-50 flex flex-col bg-background pt-[env(safe-area-inset-top)]"
		role="dialog"
		aria-modal="true"
		aria-label="Search"
	>
		<div class="flex shrink-0 items-center gap-2 border-b px-4 py-2.5">
			<label class="relative flex min-w-0 flex-1 items-center">
				<Search class="pointer-events-none absolute left-3.5 size-4 text-muted-foreground" />
				<input
					{@attach (el) => el.focus()}
					type="search"
					enterkeyhint="go"
					autocomplete="off"
					autocapitalize="off"
					spellcheck="false"
					placeholder={scope ? `Search in ${aliases.get(scope)}` : 'Search courses, papers, slides'}
					aria-label="Search"
					class="h-11 w-full rounded-full bg-muted pr-10 pl-10 text-base outline-none placeholder:text-muted-foreground [&::-webkit-search-cancel-button]:hidden"
					bind:value={() => query, (v) => palette.update((p) => ({ ...p, query: v }))}
					onkeydown={(ev) => {
						if (ev.key === 'Enter' && groups[0]) primary(groups[0].rows[0]);
						if (ev.key === 'Escape') dismiss();
					}}
				/>
				{#if query}
					<button
						type="button"
						class="absolute right-1.5 flex size-8 items-center justify-center rounded-full text-muted-foreground"
						aria-label="Clear search"
						onclick={() => palette.update((p) => ({ ...p, query: '' }))}
					>
						<X class="size-4" />
					</button>
				{/if}
			</label>
			<button type="button" class="h-11 px-1 text-[15px] font-medium" onclick={dismiss}
				>Cancel</button
			>
		</div>
		{#if scope}
			<div class="flex shrink-0 items-center gap-2 border-b px-4 py-2 text-[13px]">
				<span class="text-muted-foreground">Searching in</span>
				<button
					type="button"
					class="flex h-7 items-center gap-1.5 rounded-full bg-muted pr-2 pl-3 font-medium"
					aria-label="Search everywhere instead"
					onclick={() => palette.update((p) => ({ ...p, scope: null }))}
				>
					{aliases.get(scope)}<X class="size-3.5 text-muted-foreground" />
				</button>
			</div>
		{/if}
		<div class="min-h-0 flex-1 overflow-y-auto overscroll-contain pb-[env(safe-area-inset-bottom)]">
			{#each groups as group (group.name)}
				<div
					class="sticky top-0 bg-background/95 px-4 pt-4 pb-1.5 text-xs font-medium text-muted-foreground backdrop-blur"
				>
					{query.trim() ? group.name : 'My courses'}
				</div>
				{#each group.rows as e (e.id)}{@render row(e)}{/each}
			{/each}
			{#if !query.trim()}
				<div class="flex flex-col gap-2.5 px-4 pt-5">
					<span class="text-xs font-medium text-muted-foreground">Try</span>
					<div class="flex flex-wrap gap-2">
						{#each tries as q (q)}
							<button
								type="button"
								class="h-9 rounded-full border px-3.5 font-mono text-[13px] active:bg-accent"
								onclick={() => palette.update((p) => ({ ...p, query: q }))}>{q}</button
							>
						{/each}
					</div>
				</div>
			{:else if !groups.length}
				<div class="px-6 py-16 text-center text-sm text-muted-foreground">
					Nothing matches. Try a course alias or a folder name.
				</div>
			{/if}
		</div>
	</div>
{/snippet}

{#snippet hint(label: string, keys: string, muted = false)}
	<div class="flex items-center justify-between {muted ? 'text-muted-foreground' : ''}">
		<span>{label}</span><Kbd class="border bg-background font-mono">{keys}</Kbd>
	</div>
{/snippet}

{#if phone.current}
	{#if $palette.open}{@render fullscreen()}{/if}
{:else}
	<Dialog.Root
		bind:open={
			() => $palette.open, (v) => (v ? palette.update((p) => ({ ...p, open: v })) : closepalette())
		}
	>
		<Dialog.Content
			showCloseButton={false}
			onOpenAutoFocus={(e) => {
				e.preventDefault();
				input?.focus();
			}}
			class="gap-0 overflow-hidden rounded-2xl p-0 shadow-2xl sm:max-w-[640px]"
		>
			<Dialog.Title class="sr-only">Search</Dialog.Title>
			<Command.Root
				shouldFilter={false}
				bind:value={active}
				class="rounded-none bg-background"
				onkeydowncapture={onkeydown}
			>
				<div class="flex h-14 items-center gap-2.5 border-b px-4">
					<Search class="size-4.5 shrink-0 text-muted-foreground" />
					{#if scope}
						<button
							type="button"
							class="flex h-6.5 shrink-0 items-center gap-1.5 rounded-md bg-muted pr-1.5 pl-2 text-[13px]"
							aria-label="Remove course scope"
							onclick={() => palette.update((p) => ({ ...p, scope: null }))}
						>
							in {aliases.get(scope)}
							<X class="size-3" />
						</button>
					{/if}
					<CommandPrimitive.Input
						bind:ref={input}
						bind:value={() => query, (v) => palette.update((p) => ({ ...p, query: v }))}
						placeholder="Search courses, folders and files"
						class="min-w-0 flex-1 bg-transparent text-base outline-hidden placeholder:text-muted-foreground"
					/>
					<Kbd class="border bg-background font-mono">esc</Kbd>
				</div>
				<div class="flex h-[min(440px,60dvh)]">
					<Command.List class="max-h-none flex-1 p-2">
						{#each groups as group (group.name)}
							<Command.Group
								heading={group.name}
								class="p-0 **:data-[command-group-heading]:px-2.5 **:data-[command-group-heading]:pt-2.5 **:data-[command-group-heading]:font-normal"
							>
								{#each group.rows as e (e.id)}
									<Command.Item
										value={e.id}
										onSelect={() => primary(e)}
										class="min-h-11 gap-3 rounded-lg px-2.5 py-1.5"
									>
										<span class="w-9 shrink-0 truncate font-mono text-xs text-muted-foreground"
											>{alias(e)}</span
										>
										<span class="flex min-w-0 flex-1 flex-col gap-px">
											<span class="truncate text-sm">{e.title}</span>
											<span class="truncate font-mono text-[11px] text-muted-foreground/70"
												>{sub(e)}</span
											>
										</span>
										<span class="text-xs whitespace-nowrap text-muted-foreground/70">{tag(e)}</span>
									</Command.Item>
								{/each}
							</Command.Group>
						{/each}
						{#if !groups.length}
							<div class="px-4 py-12 text-center text-sm text-muted-foreground">
								{query.trim()
									? 'Nothing matches. Try a course alias or a folder name.'
									: 'Type to search courses, folders and files.'}
							</div>
						{/if}
					</Command.List>
					{#if detail && sel}
						<div class="hidden w-[38%] shrink-0 flex-col gap-3 bg-muted/40 p-6 md:flex">
							<div class="font-mono text-xs text-muted-foreground">
								{alias(sel)} · {sel.t === 'c' ? 'course' : sel.t === 'd' ? 'folder' : tag(sel)}
							</div>
							<div class="text-lg leading-snug font-semibold tracking-tight">{sel.title}</div>
							<div class="font-mono text-xs break-all text-muted-foreground">
								{sel.t === 'c' ? `${sel.path} · ${sel.count} files` : sel.path}
							</div>
							<div class="flex-1"></div>
							<div class="flex flex-col gap-1.5 text-[13px]">
								{#if sel.t === 'c'}
									{@render hint('Open course', '↵')}
									{@render hint(
										$mycourses.includes(sel.course!)
											? 'Remove from my courses'
											: 'Add to my courses',
										'Ctrl ↵',
										true
									)}
								{:else}
									{@render hint(
										sel.t === 'f'
											? 'Open with preview'
											: sel.course
												? 'Open in course'
												: 'Open in source',
										'↵'
									)}
									{@render hint('Open on GitHub', 'Ctrl ↵', true)}
								{/if}
								{@render hint('Copy link', 'Ctrl C', true)}
							</div>
						</div>
					{/if}
				</div>
				{#if detail}
					<div
						class="hidden h-10 items-center gap-4 border-t px-4 text-xs text-muted-foreground sm:flex"
					>
						<span
							><Kbd class="border bg-background font-mono">↑</Kbd>
							<Kbd class="border bg-background font-mono">↓</Kbd> move</span
						>
						<span><Kbd class="border bg-background font-mono">backspace</Kbd> drop scope</span>
						<span class="flex-1"></span>
						<span>ranked: this course, my courses, everything</span>
					</div>
				{/if}
			</Command.Root>
		</Dialog.Content>
	</Dialog.Root>
{/if}
