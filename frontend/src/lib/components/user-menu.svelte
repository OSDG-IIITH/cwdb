<script lang="ts">
	import { user, loading, login, logout } from '$lib/auth';
	import { mode, setMode } from 'mode-watcher';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { buttonVariants } from '$lib/components/ui/button';
	import { cn } from '$lib/utils';
	import { Gem, LogIn, LogOut, Moon, Settings, Sprout, Sun } from '@lucide/svelte';
	import { onMount } from 'svelte';

	const themes = [
		{ key: 'light', label: 'Light', icon: Sun },
		{ key: 'dark', label: 'Dark', icon: Moon },
		{ key: 'mint', label: 'Mint', icon: Sprout },
		{ key: 'amethyst', label: 'Amethyst', icon: Gem }
	];

	let variant = $state('default');
	let current = $derived(variant === 'default' ? (mode.current ?? 'light') : variant);

	/** display name from the part of an email before the @ */
	const nameof = (email: string) =>
		email
			.split('@')[0]
			.split(/[._-]/)
			.map((p) => p.charAt(0).toUpperCase() + p.slice(1).toLowerCase())
			.join(' ');

	/** two-letter initials from an email */
	const initials = (email: string) => {
		const parts = email.split('@')[0].split(/[._-]/);
		return (parts.length > 1 ? parts[0][0] + parts[1][0] : parts[0].slice(0, 2)).toUpperCase();
	};

	onMount(() => apply(localStorage.getItem('theme-variant') || 'default'));

	/** sets the colour variant class and the light or dark mode it needs */
	function apply(v: string) {
		document.documentElement.classList.remove('mint', 'amethyst');
		if (v === 'mint' || v === 'amethyst') {
			document.documentElement.classList.add(v);
			setMode(v === 'mint' ? 'light' : 'dark');
		}
		localStorage.setItem('theme-variant', v);
		variant = v;
	}

	/** picks one of the four themes */
	function pick(key: string) {
		if (key === 'light' || key === 'dark') {
			apply('default');
			setMode(key);
		} else apply(key);
	}
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger
		disabled={$loading}
		aria-label="Settings"
		class={cn(
			buttonVariants({ variant: 'ghost', size: 'icon' }),
			'size-8 rounded-full text-muted-foreground hover:text-foreground data-[state=open]:bg-accent data-[state=open]:text-foreground'
		)}
	>
		<Settings class="size-4" />
	</DropdownMenu.Trigger>

	<DropdownMenu.Content align="end" class="w-64 rounded-xl p-1.5" sideOffset={10}>
		{#if $user}
			<div class="flex items-center gap-3 px-2 py-2">
				<span
					class="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-[11px] font-medium"
					>{initials($user.email)}</span
				>
				<div class="min-w-0 flex-1">
					<p class="truncate text-sm font-medium">{nameof($user.email)}</p>
					<p class="truncate text-xs text-muted-foreground">{$user.email}</p>
				</div>
				{#if $user.role === 'admin'}
					<span class="rounded-md bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground"
						>admin</span
					>
				{/if}
			</div>
			<DropdownMenu.Separator />
		{/if}

		<div class="px-2 pt-1.5 pb-1 text-xs text-muted-foreground">Theme</div>
		<div class="grid grid-cols-4 gap-1 px-1 pb-1.5">
			{#each themes as t (t.key)}
				<button
					type="button"
					aria-pressed={current === t.key}
					class={cn(
						'flex flex-col items-center gap-1 rounded-lg border border-transparent py-2 text-[11px] text-muted-foreground transition-colors hover:bg-accent hover:text-foreground',
						current === t.key && 'border-border bg-accent text-foreground'
					)}
					onclick={() => pick(t.key)}
				>
					<t.icon class="size-4" />
					{t.label}
				</button>
			{/each}
		</div>

		<DropdownMenu.Separator />
		{#if $user}
			<DropdownMenu.Item onclick={logout} class="cursor-pointer">
				<LogOut />Log out
			</DropdownMenu.Item>
		{:else}
			<DropdownMenu.Item onclick={login} class="cursor-pointer">
				<LogIn />Log in
			</DropdownMenu.Item>
		{/if}
	</DropdownMenu.Content>
</DropdownMenu.Root>
