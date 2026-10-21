<script lang="ts">
	import { ArrowUpRight } from '@lucide/svelte';
	import { kinds, type CourseInfo } from '$lib/sync';
	import { coursehref } from '$lib/utils';

	let { c }: { c: CourseInfo } = $props();

	const terms: Record<string, string> = { monsoon: 'Monsoon', spring: 'Spring', both: 'Both terms' };
	let has = $derived(kinds.filter((k) => k !== 'other' && c.kinds.includes(k)));
</script>

<a
	href={coursehref(c.code)}
	data-code={c.code}
	class="group flex min-h-28 flex-col gap-1.5 rounded-xl border bg-card p-5 shadow-xs transition-all hover:border-foreground/20 hover:shadow-sm"
>
	<span class="flex items-center justify-between gap-3 text-xs text-muted-foreground">
		<span class="truncate font-mono">{c.alias}</span>
		<ArrowUpRight
			class="size-3.5 opacity-0 transition-opacity group-hover:opacity-100"
			aria-hidden="true"
		/>
	</span>
	<span class="line-clamp-2 text-[15px] leading-snug font-medium">{c.name}</span>
	<span class="mt-auto truncate pt-2 text-xs text-muted-foreground">
		{[terms[c.semester], `${c.count} files`, has.slice(0, 3).join(', ')]
			.filter(Boolean)
			.join(' · ')}
	</span>
</a>
