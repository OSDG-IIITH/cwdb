<script lang="ts">
	import { base } from '$app/paths';
	import { page } from '$app/state';
	import { user, loading } from '$lib/auth';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { Button, buttonVariants } from '$lib/components/ui/button';
	import ThemePicker from '$lib/components/themepicker.svelte';
	import { cn } from '$lib/utils';
	import { Settings } from '@lucide/svelte';

	const trigger =
		'size-8 rounded-full text-muted-foreground hover:text-foreground data-[state=open]:bg-accent data-[state=open]:text-foreground';
</script>

{#if $user}
	<Button
		variant="ghost"
		size="icon"
		href="{base}/settings"
		aria-label="Settings"
		title="Settings"
		aria-current={page.url.pathname === `${base}/settings` ? 'page' : undefined}
		class={cn(trigger, 'aria-[current=page]:bg-accent aria-[current=page]:text-foreground')}
	>
		<Settings class="size-4" />
	</Button>
{:else}
	<DropdownMenu.Root>
		<DropdownMenu.Trigger
			disabled={$loading}
			aria-label="Settings"
			class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), trigger)}
		>
			<Settings class="size-4" />
		</DropdownMenu.Trigger>

		<DropdownMenu.Content align="end" class="w-64 rounded-xl p-1.5" sideOffset={10}>
			<div class="px-2 pt-1.5 pb-1 text-xs text-muted-foreground">Theme</div>
			<div class="px-1 pb-1.5"><ThemePicker /></div>
		</DropdownMenu.Content>
	</DropdownMenu.Root>
{/if}
