<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { ModeWatcher } from 'mode-watcher';
	import Topbar from '$lib/components/topbar.svelte';
	import Palette from '$lib/components/palette.svelte';
	import { onMount } from 'svelte';
	import { fetchUser } from '$lib/auth';
	import { Toaster } from '$lib/components/ui/sonner';
	import { init } from '$lib/sync';
	import { initprofile } from '$lib/profile';
	import { applyvariant } from '$lib/theme';
	import { pwaInfo } from 'virtual:pwa-info';

	let { children } = $props();

	onMount(() => {
		initprofile();
		fetchUser();
		init();
		applyvariant(localStorage.getItem('theme-variant') || 'default');

		if ('serviceWorker' in navigator && !import.meta.env.DEV) {
			import('virtual:pwa-register').then(({ registerSW }) => {
				registerSW({
					immediate: true,
					onRegistered(r) {
						console.log('SW registered:', r);
					},
					onRegisterError(error) {
						console.error('SW registration error:', error);
					}
				});
			});
		}
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	{@html pwaInfo?.webManifest?.linkTag ?? ''}
</svelte:head>
<ModeWatcher />
<Toaster />
<Palette />
<div class="flex min-h-dvh flex-col font-sans">
	<Topbar />
	{@render children()}
</div>
