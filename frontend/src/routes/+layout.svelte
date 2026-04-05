<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { ModeWatcher } from 'mode-watcher';
	import UserMenu from '$lib/components/user-menu.svelte';
	import Sidebar from '$lib/components/nav/sidebar.svelte';
	import Mobilebar from '$lib/components/nav/mobilebar.svelte';
	import { onMount } from 'svelte';
	import { fetchUser } from '$lib/auth';
	import { Toaster } from '$lib/components/ui/sonner';

	let { children } = $props();

	onMount(() => {
		fetchUser();
		const v = localStorage.getItem('theme-variant') || 'default';
		if (v === 'mint') document.documentElement.classList.add('mint');
		if (v === 'amethyst') document.documentElement.classList.add('amethyst');
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
<ModeWatcher />
<Toaster />
<Sidebar />
<Mobilebar />
<div
	class="fixed top-5 right-5 z-50 md:top-auto md:right-auto md:bottom-2 md:left-4 md:rounded-md md:p-2"
>
	<UserMenu />
</div>

<div class="pb-28 md:pb-0">
	{@render children()}
</div>
