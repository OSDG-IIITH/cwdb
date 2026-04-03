<script lang="ts">
	import Check from '@lucide/svelte/icons/check';
	import SlidersHorizontal from '@lucide/svelte/icons/sliders-horizontal';
	import * as Popover from '$lib/components/ui/popover';
	import * as Command from '$lib/components/ui/command';
	import { type Course } from '$lib/api';

	interface Props {
		courses: Course[];
		selected: string | null;
	}

	let { courses, selected = $bindable(null) }: Props = $props();

	let open = $state(false);
	let activecourse = $derived(courses.find((course) => course.id === selected) ?? null);
	let orderedcourses = $derived.by(() => {
		if (!selected) return courses;
		const selectedcourse = courses.find((course) => course.id === selected);
		if (!selectedcourse) return courses;
		return [selectedcourse, ...courses.filter((course) => course.id !== selected)];
	});

	function titlecase(str: string): string {
		return str.replace(/\b\w/g, (c) => c.toUpperCase());
	}


</script>

<Popover.Root bind:open>
	<Popover.Trigger>
		{#snippet child({ props })}
			{@const { class: triggerClass, ...triggerProps } = props}
			<button
				type="button"
				aria-label="filter courses"
				class={[
					'h-12 w-12 shrink-0 inline-flex items-center justify-center rounded-md border shadow-xs transition-all outline-none',
					'focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]',
					selected
						? 'border-border bg-input/30 text-foreground'
						: 'border-border bg-input/30 text-muted-foreground hover:text-foreground',
					triggerClass
				]}
				{...triggerProps}
			>
				<SlidersHorizontal class="size-4" />
			</button>
		{/snippet}
	</Popover.Trigger>
	<Popover.Content class="p-0" align="start">
		<Command.Root>
			<!--
			<div class="border-b border-border px-3 py-2 text-xs text-muted-foreground">
				{#if activecourse}
					active: <span class="text-foreground">{titlecase(activecourse.name)}</span>
				{:else}
					active: <span class="text-foreground">all courses</span>
				{/if}
			</div>
			-->
			<Command.Input placeholder="search courses..." />
			<Command.List>
				<Command.Empty>no courses found</Command.Empty>
				<Command.Group>
					{#each orderedcourses as course (course.id)}
						<Command.Item
							value={course.name}
							onSelect={() => {
								selected = selected === course.id ? null : course.id;
								open = false;
							}}
						>
							<Check class={selected === course.id ? 'opacity-100' : 'opacity-0'} />
							{titlecase(course.name)}
							<span class="ml-auto text-muted-foreground">{course.resource_count}</span>
						</Command.Item>
					{/each}
				</Command.Group>
			</Command.List>
		</Command.Root>
	</Popover.Content>
</Popover.Root>
