<script lang="ts">
	import { mode } from 'mode-watcher';
	import { Gem, Moon, Sprout, Sun } from '@lucide/svelte';
	import { pushprefs } from '$lib/profile';
	import { picktheme, variant } from '$lib/theme';
	import { cn } from '$lib/utils';

	let { large = false }: { large?: boolean } = $props();

	const themes = [
		{ key: 'light', label: 'Light', icon: Sun },
		{ key: 'dark', label: 'Dark', icon: Moon },
		{ key: 'mint', label: 'Mint', icon: Sprout },
		{ key: 'amethyst', label: 'Amethyst', icon: Gem }
	];

	let current = $derived($variant === 'default' ? (mode.current ?? 'light') : $variant);

	/** picks one of the four themes and syncs it to the account */
	function pick(key: string) {
		picktheme(key);
		pushprefs();
	}
</script>

<div class={cn('grid grid-cols-4', large ? 'gap-2' : 'gap-1')}>
	{#each themes as t (t.key)}
		<button
			type="button"
			aria-pressed={current === t.key}
			class={cn(
				'flex flex-col items-center gap-1 rounded-lg border border-transparent py-2 text-[11px] text-muted-foreground transition-colors hover:bg-accent hover:text-foreground',
				large && 'gap-1.5 rounded-xl border-border py-3.5 text-xs',
				current === t.key && 'border-border bg-accent text-foreground'
			)}
			onclick={() => pick(t.key)}
		>
			<t.icon class="size-4" />
			{t.label}
		</button>
	{/each}
</div>
