<script lang="ts">
	import { SvelteSet } from 'svelte/reactivity';
	import { toast } from 'svelte-sonner';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Kbd } from '$lib/components/ui/kbd';
	import Choice from '$lib/components/choice.svelte';
	import {
		createoffering,
		editresources,
		loadreview,
		publish,
		refreshpending,
		reviewqueue,
		type ReviewRow
	} from '$lib/api';
	import { user } from '$lib/auth';
	import { courses, kinds, trysync } from '$lib/sync';
	import { cn } from '$lib/utils';

	const threshold = 0.7;
	const skipped = new SvelteSet<number>();
	const opt = (list: string[]) => list.map((v) => ({ value: v, label: v }));

	let admin = $derived($user?.role === 'admin');
	let rows = $derived($reviewqueue.rows.filter((r) => !skipped.has(r.id)));
	let open = $state<number | null>(null);
	let course = $state('');
	let kind = $state('');
	let sub = $state('');
	let year = $state('');
	let key = $state(false);
	let busy = $state(false);

	$effect(() => {
		if (admin) loadreview();
	});

	const suggestion = (r: ReviewRow) =>
		[r.course ?? 'Unclassified', r.kind ?? 'unclassified', r.examtype ?? r.slidetype, r.year, r.key && 'key']
			.filter(Boolean)
			.join(' · ');

	function drop(ids: number[]) {
		reviewqueue.update((q) => ({ ...q, rows: q.rows.filter((r) => !ids.includes(r.id)) }));
		open = null;
	}

	function pick(r: ReviewRow) {
		if (open === r.id) return (open = null);
		open = r.id;
		course = r.course ?? '';
		kind = r.kind ?? '';
		sub = r.examtype ?? r.slidetype ?? '';
		year = r.year?.toString() ?? '';
		key = r.key;
	}

	/** locks rows as manual with their current values */
	async function accept(ids: number[]) {
		if (!(await editresources(ids.map(String), {}))) return toast.error('could not save');
		drop(ids);
	}

	async function acceptall() {
		const ids = rows.filter((r) => (r.conf ?? 0) > threshold).map((r) => r.id);
		if (ids.length) await accept(ids);
	}

	async function save(r: ReviewRow) {
		busy = true;
		const edit: Record<string, unknown> = {
			course: course || null,
			kind: kind || null,
			examtype: kind === 'exams' ? sub || null : null,
			slidetype: kind === 'slides' ? sub || null : null,
			key
		};
		const dated = course && year && Number(year) !== r.year;
		if (dated) {
			const id = await createoffering(course, r.season ?? 'M', Number(year));
			if (!id) {
				busy = false;
				return toast.error('could not create the offering');
			}
			edit.offering = Number(id);
		} else if (!year) edit.offering = null;
		const ok = await editresources([String(r.id)], edit);
		busy = false;
		if (!ok) return toast.error('could not save');
		drop([r.id]);
	}

	async function ship() {
		if (!(await publish())) return toast.error('publish failed');
		toast.success('published');
		refreshpending();
		trysync();
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.ctrlKey || e.metaKey || e.altKey || (e.target as HTMLElement).closest('input, [role]')) return;
		const i = rows.findIndex((r) => r.id === open);
		const cur = rows[i];
		const go = (by: number) => {
			const next = rows[Math.min(rows.length - 1, Math.max(0, i + by))];
			if (next) pick(next);
		};
		const keys: Record<string, () => void> = {
			j: () => go(1),
			k: () => go(-1),
			a: () => cur && accept([cur.id]),
			e: () => cur && !busy && open === cur.id && document.getElementById('revcourse')?.focus(),
			s: () => {
				if (!cur) return;
				go(1);
				skipped.add(cur.id);
			}
		};
		if (e.key in keys) {
			e.preventDefault();
			keys[e.key]();
		}
	}
</script>

<svelte:window {onkeydown} />

{#if admin}
	<main
		class="mx-auto flex w-full max-w-[1180px] flex-col gap-5 overflow-y-auto px-4 py-8 lg:px-6"
	>
		<div class="flex flex-wrap items-end justify-between gap-4">
			<div class="flex flex-col gap-1.5">
				<h1 class="text-[26px] font-semibold tracking-tight">Review</h1>
				<p class="text-[13px] text-muted-foreground">
					<span class="font-mono text-foreground">{rows.length}</span> left from the last run · below
					0.8 confidence, or where the rule and the model disagree
				</p>
			</div>
			<div class="flex gap-2">
				<Button variant="outline" size="sm" onclick={acceptall}>Accept all above {threshold}</Button>
				<Button size="sm" onclick={ship}>Publish</Button>
			</div>
		</div>

		<div class="overflow-x-auto">
			<div class="min-w-[900px] border-t">
				<div
					class="grid h-[34px] grid-cols-[minmax(0,1fr)_90px_220px_56px_150px] items-center gap-4 border-b px-3 text-xs text-muted-foreground"
				>
					<span>path</span><span>rule</span><span>suggested</span><span class="text-right">conf</span
					><span></span>
				</div>
				{#each rows as r (r.id)}
					{@const at = r.path.lastIndexOf('/')}
					<div class={cn('border-b', open === r.id && 'bg-muted/50')}>
						<div
							class="grid min-h-[52px] grid-cols-[minmax(0,1fr)_90px_220px_56px_150px] items-center gap-4 px-3"
						>
							<button
								class="flex min-w-0 flex-col gap-0.5 text-left"
								onclick={() => pick(r)}
							>
								<span class="truncate font-mono text-xs">{r.path.slice(at + 1)}</span>
								<span class="truncate font-mono text-[11px] text-muted-foreground"
									>{r.path.slice(0, Math.max(at, 0))}</span
								>
							</button>
							<span class="text-xs text-muted-foreground">{r.rule}</span>
							<span class="flex flex-col gap-px text-[13px]">
								<span>{suggestion(r)}</span>
								<span class="text-[11px] text-muted-foreground">{r.why}</span>
							</span>
							<span
								class={cn(
									'text-right font-mono text-xs',
									(r.conf ?? 0) < 0.6 ? 'text-foreground' : 'text-muted-foreground'
								)}>{r.conf?.toFixed(2) ?? '—'}</span
							>
							<span class="flex justify-end gap-1.5">
								<Button variant="outline" size="xs" onclick={() => pick(r)}>Edit</Button>
								<Button size="xs" onclick={() => accept([r.id])}>Accept</Button>
							</span>
						</div>
						{#if open === r.id}
							<form
								class="flex flex-wrap items-end gap-3 px-3 pt-1 pb-4"
								onsubmit={(e) => {
									e.preventDefault();
									save(r);
								}}
							>
								<div class="flex w-52 flex-col gap-1.5">
									<Label for="revcourse">Course</Label>
									<Input id="revcourse" list="revcodes" bind:value={course} placeholder="CS1.301" />
									<datalist id="revcodes">
										{#each $courses as c (c.code)}<option value={c.code}>{c.name}</option>{/each}
									</datalist>
								</div>
								<div class="w-36"><Choice label="Kind" bind:value={kind} options={opt(kinds)} /></div>
								{#if kind === 'exams' || kind === 'slides'}
									<div class="w-28">
										<Choice
											label="Type"
											bind:value={sub}
											options={opt(kind === 'exams' ? ['quiz', 'mid', 'end', 'lab'] : ['lec', 'tut'])}
										/>
									</div>
								{/if}
								<div class="flex w-20 flex-col gap-1.5">
									<Label for="revyear">Year</Label>
									<Input id="revyear" type="number" bind:value={year} />
								</div>
								<div class="flex h-9 items-center gap-2">
									<Checkbox id="revkey" bind:checked={key} />
									<Label for="revkey">Answer key</Label>
								</div>
								<span class="flex-1"></span>
								<span class="text-xs text-muted-foreground"
									>saved as <span class="font-mono">by: manual</span>, never overwritten</span
								>
								<Button type="submit" size="sm" disabled={busy}>Save</Button>
							</form>
						{/if}
					</div>
				{/each}
				{#if !rows.length}
					<div class="py-14 text-center text-sm text-muted-foreground">
						Queue empty. Publish to ship the changes.
					</div>
				{/if}
			</div>
		</div>

		<div class="flex gap-4 text-xs text-muted-foreground">
			<span><Kbd>j</Kbd> <Kbd>k</Kbd> move</span><span><Kbd>a</Kbd> accept</span><span
				><Kbd>e</Kbd> edit</span
			><span><Kbd>s</Kbd> skip</span>
		</div>
	</main>
{:else}
	<div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">
		admins only
	</div>
{/if}
