<script lang="ts">
	import { togglecoursepin, toslug, type Course } from '$lib/api';
	import { base } from '$app/paths';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Pin, FileText } from '@lucide/svelte';
	import { titlecase } from '$lib/utils';

	let { course, onpintoggle }: { course: Course; onpintoggle?: () => void } = $props();

	let localpinned = $state<boolean | null>(null);
	let pinned = $derived(localpinned ?? course.pinned);

	let abbreviation = $derived((course.aliases ?? [])[0]?.toUpperCase() ?? '');

	async function handlepin(e: MouseEvent) {
		e.preventDefault();
		e.stopPropagation();
		const result = await togglecoursepin(toslug(course.name));
		if (result) {
			localpinned = result.pinned;
			onpintoggle?.();
		}
	}
</script>

<a href="{base}/courses/{toslug(course.name)}" class="block">
	<Card.Root
		class="group relative flex w-full flex-col justify-between gap-4 rounded-xl border border-border/60 bg-card/40 px-4 py-4 transition-colors hover:border-border md:px-5 md:py-5"
	>
		<div class="flex items-start justify-between gap-3">
			<div class="space-y-1 overflow-hidden">
				{#if abbreviation}
					<span class="font-mono text-xs font-medium text-muted-foreground">{abbreviation}</span>
				{/if}
				<Card.Title class="truncate text-sm font-medium tracking-tight">
					{titlecase(course.name)}
				</Card.Title>
			</div>

			<Button
				variant="ghost"
				size="icon"
				class="group/pin h-8 w-8 text-muted-foreground"
				onclick={handlepin}
			>
				<Pin
					class={`h-4 w-4 transition-colors ${pinned ? 'fill-foreground text-foreground' : 'group-hover/pin:fill-foreground/25'}`}
				/>
				<span class="sr-only">{pinned ? 'Unpin' : 'Pin'}</span>
			</Button>
		</div>

		<div class="flex items-center gap-1 text-[11px] text-muted-foreground">
			<FileText class="h-3 w-3" />
			<span>{course.resource_count} {course.resource_count === 1 ? 'resource' : 'resources'}</span>
		</div>
	</Card.Root>
</a>
