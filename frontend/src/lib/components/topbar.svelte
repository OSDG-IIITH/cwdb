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

	let pending = false;
	let timer: ReturnType<typeof setTimeout>;

	let section = $derived(page.url.pathname.slice(base.length).split('/').filter(Boolean)[0] ?? '');

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
			el.closest('input, textarea, select, [contenteditable]')
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

<svelte:window {onkeydown} />

<header
	class="sticky top-0 z-40 grid h-14 shrink-0 grid-cols-[1fr_auto_1fr] items-center gap-3 border-b bg-background/85 px-4 text-sm backdrop-blur"
>
	<a href={base || '/'} class="hidden justify-self-start font-mono text-base font-bold tracking-tight sm:block"
		>cwdb</a
	>
	<nav class="col-start-2 flex items-center gap-0.5 rounded-full border bg-muted/50 p-1">
		{#each tabs as t (t.key)}
			<a
				href="{base}/{t.key}"
				aria-current={section === t.key ? 'page' : undefined}
				class={cn(
					'flex h-8 items-center gap-2 rounded-full px-3.5 text-[13px] text-muted-foreground transition-colors hover:text-foreground',
					section === t.key && 'bg-background font-medium text-foreground shadow-xs dark:bg-accent'
				)}
			>
				<t.icon class="hidden size-3.5 sm:block" />{t.label}
			</a>
		{/each}
	</nav>
	<div class="flex items-center justify-end gap-2">
		<Button
			variant="outline"
			class="h-8 w-8 justify-start gap-2 bg-muted/60 px-2 font-normal text-muted-foreground shadow-none transition-colors hover:bg-accent hover:text-foreground lg:w-52"
			aria-label="Search everything"
			onclick={() => openpalette()}
		>
			<Search class="size-3.5" />
			<span class="hidden flex-1 text-left text-[13px] lg:inline">Search</span>
			<Kbd class="hidden border bg-background font-mono lg:inline-flex">Ctrl K</Kbd>
		</Button>
		{#if admin && $unpublished}
			<Button size="sm" variant="outline" class="h-8 font-normal" onclick={publishnow}>
				<span class="font-mono">{$unpublished}</span><span class="hidden sm:inline">unpublished · Publish</span>
			</Button>
		{/if}
		<UserMenu />
	</div>
</header>
