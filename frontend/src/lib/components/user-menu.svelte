<script lang="ts">
	import { user, loading, login, logout } from '$lib/auth';
	import { toggleMode, mode, setMode } from 'mode-watcher';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as Avatar from '$lib/components/ui/avatar';
	import { buttonVariants } from '$lib/components/ui/button';
	import { cn } from '$lib/utils';
	import { Sun, Moon, LogIn, LogOut, User, Sprout, Gem, Stone } from '@lucide/svelte';
	import { onMount } from 'svelte';


	function getNameFromEmail(email: string): string {
		const namePart = email.split('@')[0];
		return namePart
			.split(/[._-]/)
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1).toLowerCase())
			.join(' ');
	}

	function getInitials(email: string): string {
		const name = email.split('@')[0];
		const parts = name.split(/[._-]/);
		if (parts.length >= 2) {
			return (parts[0][0] + parts[1][0]).toUpperCase();
		}
		return name.slice(0, 2).toUpperCase();
	}

	let innerWidth = $state(0);
	let isDesktop = $derived(innerWidth >= 768);
	let align = $derived<'start' | 'end'>(isDesktop ? 'start' : 'end');
	let side = $derived<'top' | 'bottom'>(isDesktop ? 'top' : 'bottom');

	let themeVariant = $state('default');

	onMount(() => {
		themeVariant = localStorage.getItem('theme-variant') || 'default';
        applyThemeVariant(themeVariant);
	});

	function applyThemeVariant(v: string) {
		document.documentElement.classList.remove('mint', 'amethyst');
		if (v === 'mint') {
			document.documentElement.classList.add('mint');
			setMode('light');
		} else if (v === 'amethyst') {
			document.documentElement.classList.add('amethyst');
			setMode('dark');
		} 
		localStorage.setItem('theme-variant', v);
		themeVariant = v;
	}

    function setVariant(v: string) {
        applyThemeVariant(v);
    }

    function handleStandardLight() {
        setVariant('default');
        setMode('light');
    }

    function handleStandardDark() {
        setVariant('default');
        setMode('dark');
    }
</script>

<svelte:window bind:innerWidth />

<DropdownMenu.Root>
	{#if $loading}
		<DropdownMenu.Trigger
			disabled
			class="h-10 w-10 rounded-full bg-muted flex items-center justify-center opacity-60"
		>
			<User class="h-4 w-4 text-muted-foreground" />
		</DropdownMenu.Trigger>
	{:else if $user}
		<DropdownMenu.Trigger
			class="h-10 w-10 rounded-full ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
		>
			<Avatar.Root class="h-10 w-10 cursor-pointer">
				<Avatar.Fallback class="text-xs font-medium">{getInitials($user.email)}</Avatar.Fallback>
			</Avatar.Root>
		</DropdownMenu.Trigger>
	{:else}
		<DropdownMenu.Trigger class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'h-10 w-10')}>
			<User class="h-4 w-4" />
		</DropdownMenu.Trigger>
	{/if}

	<DropdownMenu.Content {align} {side} class="w-69 bg-sidebar" sideOffset={12}>
		{#if $user}
			<div class="px-3 py-2.5 border-b">
				<p class="text-sm font-medium leading-none">{getNameFromEmail($user.email)}</p>
				<p class="text-xs text-muted-foreground mt-1 truncate">{$user.email}</p>
			</div>
		{/if}

		<div class="p-1.5">
			<div class="flex items-center justify-between px-2 py-1 gap-2">
				<span class="text-xs font-medium text-muted-foreground mr-4">Theme</span>
                <div class="grid grid-cols-4 gap-2 w-full bg-muted/40 rounded-md p-1.5">
					<button
						type="button"
						class={cn(
							'flex items-center justify-center p-2 rounded-sm transition-all h-9 w-9',
							themeVariant === 'default' && mode.current === 'light'
								? 'bg-sidebar text-foreground shadow-sm'
								: 'text-muted-foreground hover:bg-sidebar/50'
						)}
						onclick={(e) => {
							e.preventDefault();
                            handleStandardLight();
						}}
						aria-label="Light Mode"
                        title="Light"
					>
						<Sun class="h-4 w-4" />
					</button>
                    <button
						type="button"
						class={cn(
							'flex items-center justify-center p-2 rounded-sm transition-all h-9 w-9',
							themeVariant === 'mint'
								? 'bg-sidebar text-foreground shadow-sm'
								: 'text-muted-foreground hover:bg-sidebar/50'
						)}
						onclick={(e) => {
							e.preventDefault();
							setVariant('mint');
						}}
						aria-label="Mint Theme"
                        title="Mint"
					>
						<Sprout class="h-4 w-4" />
					</button>
                    <button
						type="button"
						class={cn(
							'flex items-center justify-center p-2 rounded-sm transition-all h-9 w-9',
							themeVariant === 'amethyst'
								? 'bg-sidebar text-foreground shadow-sm'
								: 'text-muted-foreground hover:bg-sidebar/50'
						)}
						onclick={(e) => {
							e.preventDefault();
							setVariant('amethyst');
						}}
						aria-label="Amethyst Theme"
                        title="Amethyst"
					>
						<Stone class="h-4 w-4" />
					</button>
					<button
						type="button"
						class={cn(
							'flex items-center justify-center p-2 rounded-sm transition-all h-9 w-9',
							themeVariant === 'default' && mode.current === 'dark'
								? 'bg-sidebar text-foreground shadow-sm'
								: 'text-muted-foreground hover:bg-sidebar/50'
						)}
						onclick={(e) => {
							e.preventDefault();
                            handleStandardDark();
						}}
						aria-label="Dark Mode"
                        title="Dark"
					>
						<Moon class="h-4 w-4" />
					</button>
				</div>
			</div>
		</div>

		{#if $user}
			<DropdownMenu.Separator />
			<div class="p-1">
				<DropdownMenu.Item onclick={logout} class="cursor-pointer text-sm">
					<LogOut class="mr-2 h-4 w-4" />
					<span>Log out</span>
				</DropdownMenu.Item>
			</div>
		{:else}
			<DropdownMenu.Separator />
			<div class="p-1">
				<DropdownMenu.Item onclick={login} class="cursor-pointer text-sm">
					<LogIn class="mr-2 h-4 w-4" />
					<span>Log in</span>
				</DropdownMenu.Item>
			</div>
		{/if}
	</DropdownMenu.Content>
</DropdownMenu.Root>
