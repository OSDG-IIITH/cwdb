<script lang="ts">
	import { ArrowUpRight } from '@lucide/svelte';
	import type { CourseInfo } from '$lib/sync';
	import { cn, coursehref } from '$lib/utils';

	let { c, small = false }: { c: CourseInfo; small?: boolean } = $props();

	const terms: Record<string, string> = { monsoon: 'Monsoon', spring: 'Spring', both: 'Both terms' };
</script>

<a
	href={coursehref(c.code)}
	data-code={c.code}
	class={cn(
		'group flex flex-col rounded-xl border bg-card shadow-xs transition-all hover:border-foreground/20 hover:shadow-sm',
		small ? 'gap-1 px-4 py-3' : 'min-h-28 gap-1.5 p-5'
	)}
>
	<span class="flex items-center justify-between gap-3 text-xs text-muted-foreground">
		<span class="truncate font-mono">{c.alias}</span>
		<ArrowUpRight
			class="size-3.5 opacity-0 transition-opacity group-hover:opacity-100"
			aria-hidden="true"
		/>
	</span>
	<span class={cn('leading-snug font-medium', small ? 'truncate text-sm' : 'line-clamp-2 text-[15px]')}
		>{c.name}</span
	>
	<span class="mt-auto truncate pt-2 text-xs text-muted-foreground">
		{[terms[c.semester], `${c.count} files`].filter(Boolean).join(' · ')}
	</span>
</a>
