<script lang="ts">
	import { ArrowUpRight } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import MineIcon from '$lib/components/mineicon.svelte';
	import { mine, togglemine } from '$lib/profile';
	import type { CourseInfo } from '$lib/sync';
	import { cn, coursehref } from '$lib/utils';

	let {
		c,
		small = false,
		add = false
	}: { c: CourseInfo; small?: boolean; add?: boolean } = $props();

	const terms: Record<string, string> = {
		monsoon: 'Monsoon',
		spring: 'Spring',
		both: 'Both terms'
	};

	let added = $derived($mine.includes(c.code));
</script>

{#snippet card()}
	<a
		href={coursehref(c.code)}
		data-code={c.code}
		class={cn(
			'group flex flex-col rounded-xl border bg-card shadow-xs transition-all hover:border-foreground/20 hover:shadow-sm',
			small ? 'gap-1 px-4 py-3' : 'min-h-28 gap-1.5 p-5'
		)}
	>
		<span
			class={cn(
				'flex items-center justify-between gap-3 text-xs text-muted-foreground',
				add && 'pr-8'
			)}
		>
			<span class="truncate font-mono">{c.alias}</span>
			{#if !add}
				<ArrowUpRight
					class="size-3.5 opacity-0 transition-opacity group-hover:opacity-100"
					aria-hidden="true"
				/>
			{/if}
		</span>
		<span
			class={cn(
				'leading-snug font-medium',
				small ? 'truncate text-sm' : 'line-clamp-2 text-[15px]'
			)}>{c.name}</span
		>
		<span class="mt-auto truncate pt-2 text-xs text-muted-foreground">
			{[terms[c.semester], `${c.count} files`].filter(Boolean).join(' · ')}
		</span>
	</a>
{/snippet}

{#if add}
	<div class="relative">
		{@render card()}
		<Button
			variant="ghost"
			size="icon-sm"
			class="absolute top-3 right-3 text-muted-foreground hover:text-foreground"
			aria-label={added ? `Remove ${c.alias} from my courses` : `Add ${c.alias} to my courses`}
			title={added ? 'Remove from my courses' : 'Add to my courses'}
			onclick={() => togglemine(c.code)}
		>
			<MineIcon {added} />
		</Button>
	</div>
{:else}
	{@render card()}
{/if}
