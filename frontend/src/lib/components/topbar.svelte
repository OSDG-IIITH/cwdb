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
	import { syncstatus, allresources, courses, getcourse, trysync } from '$lib/sync';
	import { toast } from 'svelte-sonner';
	import { cn } from '$lib/utils';

	const titles: Record<string, string> = {
		courses: 'Courses',
		sources: 'Sources',
		resources: 'Resources'
	};
	const jumps: Record<string, string> = { h: '', c: '/courses', s: '/sources' };

	let pending = false;
	let timer: ReturnType<typeof setTimeout>;

	let home = $derived(page.url.pathname === base || page.url.pathname === `${base}/`);

	let crumbs = $derived.by(() => {
		const section = page.url.pathname.slice(base.length).split('/').filter(Boolean)[0];
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

	let status = $derived(
		$syncstatus.syncing ? 'syncing' : $syncstatus.online ? 'synced' : 'offline'
	);

	let tip = $derived(
		`${status}${$syncstatus.synced_at ? ` · ${ago($syncstatus.synced_at)}` : ''} · ${$allresources.length} files`
	);

	/** relative time like "5m ago" */
	function ago(d: Date): string {
		const diff = Date.now() - new Date(d).getTime();
		if (diff < 60000) return 'just now';
		if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
		if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
		return new Date(d).toLocaleDateString();
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
		home ? 'justify-end gap-5' : 'gap-2.5 border-b'
	)}
>
	{#if home}
		<a href="{base}/courses" class="text-muted-foreground hover:text-foreground">Courses</a>
		<a href="{base}/sources" class="text-muted-foreground hover:text-foreground">Sources</a>
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
		<Button
			variant="outline"
			class="h-8 w-8 justify-start gap-2 bg-muted/40 px-2 font-normal text-muted-foreground sm:w-60"
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
	<span class="flex items-center gap-1.5 text-xs text-muted-foreground" title={tip} role="status">
		<span
			class={cn(
				'size-1.5 rounded-full',
				status === 'offline' ? 'border border-muted-foreground' : 'bg-foreground',
				status === 'syncing' && 'animate-pulse'
			)}
		></span>
		<span class={home ? '' : 'sr-only'}>{status}</span>
	</span>
	<UserMenu />
</header>
