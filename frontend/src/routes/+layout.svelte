<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { ModeWatcher } from 'mode-watcher';
	import UserMenu from '$lib/components/user-menu.svelte';
	import Sidebar from '$lib/components/nav/sidebar.svelte';
	import { onMount } from 'svelte';
	import { fetchUser } from '$lib/auth';

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
<Sidebar />
<div class="fixed top-5 right-5 z-50 md:top-auto md:bottom-5 md:left-5 md:right-auto">
	<UserMenu />
</div>

{@render children()}
