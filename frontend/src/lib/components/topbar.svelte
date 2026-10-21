<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { Search } from '@lucide/svelte';
	import * as Breadcrumb from '$lib/components/ui/breadcrumb';
	import { Button } from '$lib/components/ui/button';
	import { Kbd } from '$lib/components/ui/kbd';
	import UserMenu from '$lib/components/user-menu.svelte';
	import { pending as unpublished, publish, refreshpending } from '$lib/api';
	import { user } from '$lib/auth';
	import { palette, openpalette, closepalette } from '$lib/components/palette.svelte';
	import { courses, getcourse, trysync } from '$lib/sync';
	import { toast } from 'svelte-sonner';
	import { cn } from '$lib/utils';

	const titles: Record<string, string> = {
		courses: 'Courses',
		sources: 'Sources'
	};
	const jumps: Record<string, string> = { h: '', c: '/courses', s: '/sources' };

	let pending = false;
	let timer: ReturnType<typeof setTimeout>;

	let home = $derived(page.url.pathname === base || page.url.pathname === `${base}/`);

	let section = $derived(page.url.pathname.slice(base.length).split('/').filter(Boolean)[0]);

	let crumbs = $derived.by(() => {
		const out: { label: string; href: string }[] = [];
		if (section) out.push({ label: titles[section] ?? section, href: `${base}/${section}` });
		const slug = page.params.slug;
		if (section === 'courses' && slug) {
			out.push({
				label:
					slug === 'unclassified' ? 'Unclassified' : (getcourse($courses, slug)?.alias ?? slug),
				href: `${base}/courses/${encodeURIComponent(slug)}`
			});
		}
		if (section === 'sources' && page.params.owner) {
			const root = `${base}/sources/${page.params.owner}/${page.params.repo}`;
			out.push({ label: `${page.params.owner}/${page.params.repo}`, href: root });
			if (page.params.path)
				out.push({ label: page.params.path, href: `${root}/${page.params.path}` });
		}
		if (page.route.id === '/sources/review')
			out.push({ label: 'Review', href: `${base}/sources/review` });
		return out;
	});

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
	class={cn(
		'sticky top-0 z-40 flex h-12 shrink-0 items-center bg-background px-4 text-sm',
		home ? 'justify-end gap-2' : 'gap-2.5 border-b'
	)}
>
	{#snippet links()}
		<nav class="hidden items-center gap-0.5 sm:flex">
			{#each Object.entries(titles) as [key, label] (key)}
				<a
					href="{base}/{key}"
					class={cn(
						'rounded-md px-2.5 py-1 text-[13px] text-muted-foreground transition-colors hover:bg-accent hover:text-foreground',
						section === key && 'text-foreground'
					)}>{label}</a
				>
			{/each}
		</nav>
	{/snippet}
	{#if home}
		{@render links()}
	{:else}
		<Breadcrumb.Root class="min-w-0">
			<Breadcrumb.List class="flex-nowrap text-sm">
				<Breadcrumb.Item>
					<Breadcrumb.Link href={base || '/'} class="font-mono font-semibold text-foreground"
						>cwdb</Breadcrumb.Link
					>
				</Breadcrumb.Item>
				{#each crumbs as crumb, i (crumb.href)}
					<Breadcrumb.Separator class="text-border">/</Breadcrumb.Separator>
					<Breadcrumb.Item class="min-w-0">
						{#if i === crumbs.length - 1}
							<Breadcrumb.Page class="truncate font-medium">{crumb.label}</Breadcrumb.Page>
						{:else}
							<Breadcrumb.Link href={crumb.href}>{crumb.label}</Breadcrumb.Link>
						{/if}
					</Breadcrumb.Item>
				{/each}
			</Breadcrumb.List>
		</Breadcrumb.Root>
		<span class="flex-1"></span>
		{@render links()}
		<Button
			variant="outline"
			class="h-8 w-8 justify-start gap-2 bg-muted/60 px-2 font-normal text-muted-foreground shadow-none transition-colors hover:bg-accent hover:text-foreground sm:w-56"
			aria-label="Search everything"
			onclick={() => openpalette()}
		>
			<Search class="size-3.5" />
			<span class="hidden flex-1 text-left text-[13px] sm:inline">Search</span>
			<Kbd class="hidden border bg-background font-mono sm:inline-flex">Ctrl K</Kbd>
		</Button>
	{/if}
	{#if admin && $unpublished}
		<Button size="sm" variant="outline" class="h-7 font-normal" onclick={publishnow}>
			<span class="font-mono">{$unpublished}</span> unpublished · Publish
		</Button>
	{/if}
	<UserMenu />
</header>
