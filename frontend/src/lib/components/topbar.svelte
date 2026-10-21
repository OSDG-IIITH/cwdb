<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { FolderGit, House, Library, Search } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Kbd } from '$lib/components/ui/kbd';
	import UserMenu from '$lib/components/user-menu.svelte';
	import { pending as unpublished, publish, refreshpending } from '$lib/api';
	import { user } from '$lib/auth';
	import { palette, openpalette, closepalette } from '$lib/components/palette.svelte';
	import { trysync } from '$lib/sync';
	import { toast } from 'svelte-sonner';
	import { cn } from '$lib/utils';

	const tabs = [
		{ key: '', label: 'Home', icon: House },
		{ key: 'courses', label: 'Courses', icon: Library },
		{ key: 'sources', label: 'Sources', icon: FolderGit }
	];
	const jumps: Record<string, string> = { h: '', c: '/courses', s: '/sources' };

	const field = 'input, textarea, select, [contenteditable]';

	let pending = false;
	let typing = $state(false);
	let timer: ReturnType<typeof setTimeout>;

	let section = $derived(page.url.pathname.slice(base.length).split('/').filter(Boolean)[0] ?? '');
	let index = $derived(tabs.findIndex((t) => t.key === section));

	let admin = $derived($user?.role === 'admin');

	$effect(() => {
		if (admin) refreshpending();
	});

	async function publishnow() {
		const done = await publish();
		if (!done) return toast.error('publish failed');
		toast.success('published');
		refreshpending();
		trysync();
	}

	/** global keys: ctrl k and / for search, g h, g c, g s to jump */
	function onkeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
			e.preventDefault();
			if ($palette.open) closepalette();
			else openpalette();
			return;
		}
		const el = e.target as HTMLElement;
		if (
			e.ctrlKey ||
			e.metaKey ||
			e.altKey ||
			el.closest(field)
		)
			return;
		if (e.key === '/') {
			e.preventDefault();
			const filter = document.querySelector<HTMLElement>('[data-slash]');
			if (filter) filter.focus();
			else openpalette();
			return;
		}
		if (pending) {
			pending = false;
			if (e.key in jumps) {
				e.preventDefault();
				goto(`${base}${jumps[e.key]}` || '/');
			}
			return;
		}
		if (e.key === 'g') {
			pending = true;
			clearTimeout(timer);
			timer = setTimeout(() => (pending = false), 1000);
		}
	}
</script>

<svelte:window
	{onkeydown}
	onfocusin={(e) => (typing = !!(e.target as HTMLElement).closest?.(field))}
	onfocusout={() => (typing = false)}
/>

{#snippet tabsnav(big: boolean)}
	<span
		class={cn(
			'absolute left-[3px] rounded-md border border-transparent bg-background shadow-sm transition-transform duration-200 ease-out dark:border-input dark:bg-input/30',
			big ? 'inset-y-1 left-1' : 'inset-y-[3px]',
			index < 0 && 'opacity-0'
		)}
		style="width: calc((100% - {big ? 8 : 6}px) / {tabs.length}); transform: translateX({Math.max(
			index,
			0
		) * 100}%)"
	></span>
	{#each tabs as t (t.key)}
		<a
			href="{base}/{t.key}"
			aria-current={section === t.key ? 'page' : undefined}
			class={cn(
				'relative flex items-center justify-center gap-2 rounded-md px-3.5 text-sm font-medium text-muted-foreground transition-colors hover:text-foreground',
				big ? 'h-11 px-4' : 'h-full',
				section === t.key && 'text-foreground'
			)}
		>
			<t.icon class="size-4" />{t.label}
		</a>
	{/each}
{/snippet}

<header
	class={cn(
		'sticky top-0 z-40 grid h-15 shrink-0 grid-cols-[1fr_auto_1fr] items-center gap-3 bg-background/85 px-4 pt-1 text-sm backdrop-blur sm:px-8 lg:px-12',
		section && 'border-b'
	)}
>
	<a
		href={base || '/'}
		class={cn(
			'justify-self-start font-mono text-base font-bold tracking-tight',
			!section && 'sm:hidden'
		)}>cwdb</a
	>
	<nav
		aria-label="Sections"
		class="relative col-start-2 hidden h-9 auto-cols-fr grid-flow-col rounded-lg bg-muted p-[3px] sm:grid"
	>
		{@render tabsnav(false)}
	</nav>
	<div class="col-start-3 flex items-center justify-end gap-2">
		<Button
			variant="outline"
			class="size-8 justify-start gap-2 rounded-full border-transparent bg-transparent px-2 font-normal text-muted-foreground shadow-none transition-colors hover:bg-accent hover:text-foreground sm:rounded-md sm:border-input sm:bg-muted/60 lg:w-52"
			aria-label="Search everything"
			onclick={() => openpalette()}
		>
			<Search class="size-4 sm:size-3.5" />
			<span class="hidden flex-1 text-left text-[13px] lg:inline">Search</span>
			<Kbd class="hidden border bg-background font-mono lg:inline-flex">Ctrl K</Kbd>
		</Button>
		{#if admin && $unpublished}
			<Button size="sm" variant="outline" class="h-8 font-normal" onclick={publishnow}>
				<span class="font-mono">{$unpublished}</span><span class="hidden sm:inline"
					>unpublished · Publish</span
				>
			</Button>
		{/if}
		<UserMenu />
	</div>
</header>

<nav
	aria-label="Sections"
	class={cn(
		'fixed inset-x-0 bottom-[calc(env(safe-area-inset-bottom)+0.75rem)] z-40 mx-auto grid w-fit auto-cols-fr grid-flow-col rounded-xl border bg-background/90 p-1 shadow-lg backdrop-blur transition-[translate,opacity] duration-200 sm:hidden',
		typing && 'pointer-events-none translate-y-4 opacity-0'
	)}
>
	{@render tabsnav(true)}
</nav>
