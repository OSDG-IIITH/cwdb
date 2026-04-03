<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { Search, Archive, FolderOpen, BookOpen } from '@lucide/svelte';

	const links = [
		{ href: '/', icon: Search, label: 'Search' },
		{ href: '/resources', icon: FolderOpen, label: 'Resources' },
		{ href: '/courses', icon: BookOpen, label: 'Courses' },
		{ href: '/sources', icon: Archive, label: 'Sources' }
	];

	const activeIndex = $derived(
		links.findIndex(link => 
            link.href === '/' 
                ? page.url.pathname === '/' 
                : page.url.pathname.startsWith(link.href)
        )
	);
	const currentIdx = $derived(activeIndex === -1 ? 0 : activeIndex);
	$effect(() => {
		const handleKeydown = (e: KeyboardEvent) => {
			const activeTag = document.activeElement?.tagName;
			if (activeTag === 'INPUT' || activeTag === 'TEXTAREA') return;

			if (e.key === '1') {
				e.preventDefault();
				goto('/');
			} else if (e.key === '2') {
                goto('/resources');
            } else if (e.key === '3') {
				goto('/courses');
			} else if (e.key === '4') {
				goto('/sources');
			}
		};

		window.addEventListener('keydown', handleKeydown);
		
		return () => window.removeEventListener('keydown', handleKeydown);
	});
</script>

<nav class="fixed left-4 top-1/2 z-50 -translate-y-1/2 hidden md:block">
	<div class="flex flex-col gap-2 rounded-md border border-border/50 bg-card/30 p-2 shadow-lg backdrop-blur-md">
		<div 
			class="absolute left-2 w-10 h-10 rounded-lg bg-primary/10 transition-all duration-300 ease-[cubic-bezier(0.23,1,0.32,1)]"
			style="top: calc(0.5rem + {currentIdx} * 3rem);"
		></div>

		{#each links as link}
			<a
				href={link.href}
				class="relative z-10 flex h-10 w-10 items-center justify-center rounded-full text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
				class:text-foreground={link.href === '/' ? page.url.pathname === '/' : page.url.pathname.startsWith(link.href)}
				title={link.label}
			>
				<link.icon class="h-4.5 w-4.5" />
				<span class="sr-only">{link.label}</span>
			</a>
		{/each}
	</div>
</nav>
