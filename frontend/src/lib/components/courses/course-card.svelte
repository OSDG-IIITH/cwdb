<script lang="ts">
	import { base } from '$app/paths';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Pin, FileText } from '@lucide/svelte';
	import { titlecase, toslug } from '$lib/utils';
	import type { CourseInfo } from '$lib/sync';

	let { course, onpintoggle }: { course: CourseInfo; onpintoggle?: () => void } = $props();

	let pinned = $state(ispinned());

	function ispinned(): boolean {
		if (typeof localStorage === 'undefined') return false;
		const pins: string[] = JSON.parse(localStorage.getItem('pins') ?? '[]');
		return pins.includes(course.id);
	}

	function handlepin(e: MouseEvent) {
		e.preventDefault();
		e.stopPropagation();
		const pins: string[] = JSON.parse(localStorage.getItem('pins') ?? '[]');
		const idx = pins.indexOf(course.id);
		if (idx >= 0) pins.splice(idx, 1);
		else pins.push(course.id);
		localStorage.setItem('pins', JSON.stringify(pins));
		pinned = idx < 0;
		onpintoggle?.();
	}

	let abbreviation = $derived((course.aliases ?? [])[0]?.toUpperCase() ?? '');
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
