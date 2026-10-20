<script lang="ts">
	import type { Snippet } from 'svelte';
	import { goto } from '$app/navigation';
	import {
		Copy,
		ExternalLink,
		Link,
		Pin,
		PinOff,
		Search,
		SquareArrowOutUpRight
	} from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import * as ContextMenu from '$lib/components/ui/context-menu';
	import { openpalette } from '$lib/components/palette.svelte';
	import { courses, mycourses, togglepin, getcourse } from '$lib/sync';
	import { coursehref } from '$lib/utils';

	let { children }: { children: Snippet } = $props();

	let code = $state<string | null>(null);
	let course = $derived(code ? getcourse($courses, code) : null);
	let pinned = $derived(!!code && $mycourses.includes(code));

	/** picks the course under the cursor, or falls back to the native menu */
	function oncontextmenu(e: MouseEvent) {
		code = (e.target as HTMLElement).closest<HTMLElement>('[data-code]')?.dataset.code ?? null;
		if (!code) e.stopPropagation();
	}

	async function copy(text: string, what: string) {
		await navigator.clipboard.writeText(text);
		toast.success(`${what} copied`);
	}
</script>

<ContextMenu.Root>
	<ContextMenu.Trigger>
		{#snippet child({ props })}
			<div {...props} oncontextmenucapture={oncontextmenu}>{@render children()}</div>
		{/snippet}
	</ContextMenu.Trigger>
	{#if course}
		<ContextMenu.Content class="w-56">
			<ContextMenu.Label class="truncate font-mono text-xs font-normal text-muted-foreground">
				{course.alias} · {course.code}
			</ContextMenu.Label>
			<ContextMenu.Item onSelect={() => goto(coursehref(course.code))}>
				<SquareArrowOutUpRight />Open
			</ContextMenu.Item>
			<ContextMenu.Item onSelect={() => window.open(coursehref(course.code), '_blank')}>
				<ExternalLink />Open in new tab
			</ContextMenu.Item>
			<ContextMenu.Item onSelect={() => openpalette('', course.code)}>
				<Search />Search in course
			</ContextMenu.Item>
			<ContextMenu.Separator />
			<ContextMenu.Item onSelect={() => togglepin(course.code)}>
				{#if pinned}<PinOff />Remove from my courses{:else}<Pin />Add to my courses{/if}
			</ContextMenu.Item>
			<ContextMenu.Separator />
			<ContextMenu.Item onSelect={() => copy(location.origin + coursehref(course.code), 'Link')}>
				<Link />Copy link
			</ContextMenu.Item>
			<ContextMenu.Item onSelect={() => copy(course.code, 'Code')}>
				<Copy />Copy code
			</ContextMenu.Item>
		</ContextMenu.Content>
	{/if}
</ContextMenu.Root>
