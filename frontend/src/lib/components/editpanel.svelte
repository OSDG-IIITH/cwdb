<script lang="ts">
	import { untrack } from 'svelte';
	import { toast } from 'svelte-sonner';
	import * as Sheet from '$lib/components/ui/sheet';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import Choice from '$lib/components/choice.svelte';
	import { createoffering, editresources, refreshpending } from '$lib/api';
	import {
		addoffering,
		allresources,
		courses,
		kinds,
		offeringsfor,
		patchresources,
		type Resource
	} from '$lib/sync';

	let {
		ids,
		initial = null,
		oncancel
	}: { ids: string[]; initial?: Resource | null; oncancel: () => void } = $props();

	const exams = ['quiz', 'mid', 'end', 'lab'];
	const slides = ['lec', 'tut'];
	const opt = (list: string[]) => list.map((v) => ({ value: v, label: v }));

	const was = untrack(() => initial);

	let course = $state(was?.course_id ?? '');
	let offering = $state(was?.offering ?? '');
	let kind = $state(was?.kind ?? '');
	let examtype = $state(was?.examtype ?? '');
	let slidetype = $state(was?.slidetype ?? '');
	let n = $state(was?.n?.toString() ?? '');
	let key = $state(was?.key ?? false);
	let keyfor = $state(was?.keyfor ?? '');
	let title = $state(was?.custom ?? '');
	let hidden = $state(was?.hidden ?? false);
	let season = $state('M');
	let year = $state(String(new Date().getFullYear()));
	let saving = $state(false);

	let offerings = $derived(
		offeringsfor(course).map((o) => ({
			value: o.id,
			label: [o.label, o.faculty.join(', ')].filter(Boolean).join(' · ')
		}))
	);
	let papers = $derived(
		$allresources
			.filter((r) => r.course_id === course && !r.key && r.id !== was?.id)
			.map((r) => ({ value: r.id, label: r.title }))
	);

	/** empties every classification field and locks the rows */
	async function clear() {
		const blank = {
			course: null,
			offering: null,
			kind: null,
			examtype: null,
			slidetype: null,
			n: null,
			key: false,
			keyfor: null
		};
		if (!(await editresources(ids, blank))) return toast.error('could not clear');
		patchresources(ids, {
			course_id: null,
			offering: null,
			kind: null,
			examtype: null,
			slidetype: null,
			n: null,
			key: false,
			keyfor: null
		});
		toast.success('cleared');
		oncancel();
	}

	/** unlocks the rows and reruns the rules on them */
	async function reset() {
		if (!(await editresources(ids, { reset: true }))) return toast.error('could not reset');
		refreshpending();
		toast.success('reset to rules, publish to update the index');
		oncancel();
	}

	/** sends the changed fields and mirrors them in the local index */
	async function save() {
		const none = (v: string) => v || null;
		const api: Record<string, unknown> = {};
		const local: Partial<Resource> = {};

		if (course !== (was?.course_id ?? '')) {
			api.course = none(course);
			local.course_id = none(course);
		}
		let off = offering;
		saving = true;
		if (off === 'new') {
			const id = await createoffering(course, season, Number(year));
			if (!id) {
				saving = false;
				return toast.error('could not create the offering');
			}
			addoffering(id, course, season, Number(year));
			off = id;
		}
		if (off !== (was?.offering ?? '')) {
			api.offering = off ? Number(off) : null;
			local.offering = none(off);
		}
		const typed =
			[kind, examtype, slidetype].join() !==
			[was?.kind, was?.examtype, was?.slidetype].map((v) => v ?? '').join();
		if (typed) {
			api.kind = none(kind);
			api.examtype = kind === 'exams' ? none(examtype) : null;
			api.slidetype = kind === 'slides' ? none(slidetype) : null;
			local.kind = none(kind);
			local.examtype = api.examtype as string | null;
			local.slidetype = api.slidetype as string | null;
		}
		if (n !== (was?.n?.toString() ?? '')) {
			api.n = n ? Number(n) : null;
			local.n = n ? Number(n) : null;
		}
		if (was) {
			if (key !== was.key) api.key = local.key = key;
			if (hidden !== was.hidden) api.hidden = local.hidden = hidden;
			if (keyfor !== (was.keyfor ?? '')) {
				api.keyfor = keyfor ? Number(keyfor) : null;
				local.keyfor = none(keyfor);
			}
			if (title.trim() !== (was.custom ?? '')) {
				api.title = none(title.trim());
				local.custom = none(title.trim());
			}
		}

		if (Object.keys(api).length && (await editresources(ids, api))) {
			patchresources(ids, local);
			toast.success('saved');
		} else if (Object.keys(api).length) toast.error('could not save');
		saving = false;
		oncancel();
	}
</script>

<Sheet.Root open onOpenChange={(o) => !o && oncancel()}>
	<Sheet.Content side="right" class="w-full gap-0 overflow-y-auto sm:max-w-md">
		<Sheet.Header>
			<Sheet.Title>{initial ? 'Edit metadata' : `Move ${ids.length} files`}</Sheet.Title>
			<Sheet.Description class="truncate font-mono text-xs">
				{initial?.file_path ?? 'changes apply to every selected file'}
			</Sheet.Description>
		</Sheet.Header>
		<form
			class="flex flex-col gap-4 px-4 pb-4"
			onsubmit={(e) => {
				e.preventDefault();
				save();
			}}
		>
			<div class="flex flex-col gap-1.5">
				<Label for="course">Course</Label>
				<Input id="course" list="codes" bind:value={course} placeholder="CS1.301" />
				<datalist id="codes">
					{#each $courses as c (c.code)}<option value={c.code}>{c.name}</option>{/each}
				</datalist>
			</div>
			<Choice
				label="Offering"
				bind:value={offering}
				options={[...offerings, { value: 'new', label: 'New offering…' }]}
			/>
			{#if offering === 'new'}
				<div class="flex gap-2">
					<div class="flex-1">
						<Choice
							label="Season"
							bind:value={season}
							options={[
								{ value: 'M', label: 'Monsoon' },
								{ value: 'S', label: 'Spring' }
							]}
						/>
					</div>
					<div class="flex flex-1 flex-col gap-1.5">
						<Label for="year">Year</Label>
						<Input id="year" type="number" bind:value={year} />
					</div>
				</div>
			{/if}
			<Choice label="Kind" bind:value={kind} options={opt(kinds)} />
			{#if kind === 'exams'}
				<Choice label="Exam type" bind:value={examtype} options={opt(exams)} />
			{:else if kind === 'slides'}
				<Choice label="Slide type" bind:value={slidetype} options={opt(slides)} />
			{/if}
			<div class="flex flex-col gap-1.5">
				<Label for="n">Number</Label>
				<Input id="n" type="number" bind:value={n} />
			</div>
			{#if initial}
				<div class="flex flex-col gap-1.5">
					<Label for="title">Title</Label>
					<Input id="title" bind:value={title} placeholder="shown instead of the generated title" />
				</div>
				<div class="flex items-center gap-2">
					<Checkbox id="key" bind:checked={key} />
					<Label for="key">This file is an answer key</Label>
				</div>
				{#if key}
					<Choice label="Key for" bind:value={keyfor} options={papers} />
				{/if}
				<div class="flex items-center gap-2">
					<Checkbox id="hidden" bind:checked={hidden} />
					<Label for="hidden">Hidden from students</Label>
				</div>
			{/if}
			<Button type="submit" disabled={saving}>Save</Button>
			<div class="flex gap-2">
				<Button type="button" variant="outline" class="flex-1" onclick={clear}>Clear metadata</Button>
				<Button type="button" variant="outline" class="flex-1" onclick={reset}>Reset to rules</Button>
			</div>
		</form>
	</Sheet.Content>
</Sheet.Root>
