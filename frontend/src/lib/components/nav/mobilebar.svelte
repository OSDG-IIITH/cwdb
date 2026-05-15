<script lang="ts">
	import { page } from '$app/state';
	import { base } from '$app/paths';
	import { Search, Archive, FolderOpen, BookOpen } from '@lucide/svelte';

	const links = [
		{ href: base || '/', icon: Search, label: 'Search' },
		{ href: `${base}/resources`, icon: FolderOpen, label: 'Resources' },
		{ href: `${base}/courses`, icon: BookOpen, label: 'Courses' },
		{ href: `${base}/sources`, icon: Archive, label: 'Sources' }
	];

	const activeindex = $derived(
		links.findIndex((link) =>
			link.href === (base || '/')
				? page.url.pathname === base || page.url.pathname === base + '/'
				: page.url.pathname.startsWith(link.href)
		)
	);
	const currentidx = $derived(activeindex === -1 ? 0 : activeindex);
</script>

<nav
	class="fixed left-1/2 z-50 -translate-x-1/2 md:hidden"
	style="bottom: max(1rem, env(safe-area-inset-bottom));"
>
	<div
		class="relative flex gap-2 rounded-md border border-border/50 bg-card/30 p-2 shadow-lg backdrop-blur-md"
	>
		<div
			class="absolute top-2 h-10 w-10 rounded-lg bg-primary/10 transition-all duration-300 ease-[cubic-bezier(0.23,1,0.32,1)]"
			style="left: calc(0.5rem + {currentidx} * 3rem);"
		></div>

		{#each links as link (link.href)}
			<a
				href={link.href}
				class="relative z-10 flex h-10 w-10 items-center justify-center rounded-full text-muted-foreground transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
				class:text-foreground={link.href === (base || '/')
					? page.url.pathname === base || page.url.pathname === base + '/'
					: page.url.pathname.startsWith(link.href)}
				title={link.label}
			>
				<link.icon class="h-4.5 w-4.5" />
				<span class="sr-only">{link.label}</span>
			</a>
		{/each}
	</div>
</nav>
