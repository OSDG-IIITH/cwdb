<script lang="ts">
	import { Command as CommandPrimitive } from 'bits-ui';
	import { Check, Plus, Search } from '@lucide/svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Command from '$lib/components/ui/command';
	import { Kbd } from '$lib/components/ui/kbd';
	import { courses, type CourseInfo } from '$lib/sync';
	import { mine, togglemine } from '$lib/profile';

	let { open = $bindable(false) }: { open: boolean } = $props();

	let query = $state('');
	let input = $state<HTMLInputElement | null>(null);
	/** courses matching the query like the courses filter, exact and leading aliases first */
	let shown = $derived.by(() => {
		const q = query.trim().toLowerCase();
		const rank = (c: CourseInfo) => {
			const names = [c.alias, ...c.shortnames].map((s) => s.toLowerCase());
			return names.includes(q) ? 0 : names.some((s) => s.startsWith(q)) ? 1 : 2;
		};
		return $courses
			.filter(
				(c) =>
					(c.count > 0 || $mine.includes(c.code)) &&
					(!q || [c.name, c.code, ...c.shortnames].some((s) => s.toLowerCase().includes(q)))
			)
			.sort((a, b) => (q ? rank(a) - rank(b) : 0));
	});
</script>

<Dialog.Root bind:open onOpenChange={(o) => !o && (query = '')}>
	<Dialog.Content
		showCloseButton={false}
		onOpenAutoFocus={(e) => {
			e.preventDefault();
			input?.focus();
		}}
		class="gap-0 overflow-hidden rounded-2xl p-0 shadow-2xl sm:max-w-[560px]"
	>
		<Dialog.Title class="sr-only">Add a course</Dialog.Title>
		<Command.Root shouldFilter={false} class="rounded-none bg-background">
			<div class="flex h-14 items-center gap-2.5 border-b px-4">
				<Search class="size-4.5 shrink-0 text-muted-foreground" />
				<CommandPrimitive.Input
					bind:ref={input}
					bind:value={query}
					placeholder="Add a course by name, code or alias"
					class="min-w-0 flex-1 bg-transparent text-base outline-hidden placeholder:text-muted-foreground"
				/>
				<Kbd class="border bg-background font-mono">esc</Kbd>
			</div>
			<Command.List class="h-[min(440px,60dvh)] max-h-none p-2">
				{#if !shown.length}
					<div class="py-12 text-center text-sm text-muted-foreground">No courses match.</div>
				{/if}
				{#each shown as c (c.code)}
					{@const added = $mine.includes(c.code)}
					<Command.Item
						value={c.code}
						onSelect={() => togglemine(c.code)}
						class="group min-h-11 gap-3 rounded-lg px-2.5 py-1.5"
					>
						<span class="w-12 shrink-0 truncate font-mono text-xs text-muted-foreground"
							>{c.alias}</span
						>
						<span class="flex min-w-0 flex-1 flex-col gap-px">
							<span class="truncate text-sm">{c.name}</span>
							<span class="truncate font-mono text-[11px] text-muted-foreground/70"
								>{c.code} · {c.count} files</span
							>
						</span>
						{#if added}
							<Check class="text-foreground" aria-label="In my courses" />
						{:else}
							<Plus class="opacity-0 group-aria-selected:opacity-100" aria-hidden="true" />
						{/if}
					</Command.Item>
				{/each}
			</Command.List>
			<div class="flex h-10 items-center gap-4 border-t px-4 text-xs text-muted-foreground">
				<span><Kbd class="border bg-background font-mono">↵</Kbd> add or remove</span>
				<span class="flex-1"></span>
				<span>{$mine.length} in my courses</span>
			</div>
		</Command.Root>
	</Dialog.Content>
</Dialog.Root>
