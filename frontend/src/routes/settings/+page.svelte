<script lang="ts">
	import { LogIn } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Switch } from '$lib/components/ui/switch';
	import ThemePicker from '$lib/components/themepicker.svelte';
	import { user, loading, login, nameof } from '$lib/auth';
	import { profile, setautoterm, setsettings } from '$lib/profile';
	import { almanac } from '$lib/sync';
	import { phase, shortdate, ug1until } from '$lib/terms';

	const now = new Date();

	let p = $derived($profile);
	let next = $derived(p ? phase($almanac, now, p.ug1) : null);

	const month = (d: string | Date) =>
		new Date(d).toLocaleDateString('en-IN', { month: 'short', year: 'numeric' });
</script>

{#snippet toggle(
	id: string,
	label: string,
	note: string,
	checked: boolean,
	onchange: (v: boolean) => void
)}
	<div class="flex items-start justify-between gap-6 py-4">
		<label for={id} class="flex min-w-0 flex-col gap-1">
			<span class="text-sm font-medium">{label}</span>
			<span class="text-[13px] leading-relaxed text-muted-foreground">{note}</span>
		</label>
		<Switch {id} class="mt-0.5" {checked} onCheckedChange={onchange} />
	</div>
{/snippet}

<main
	class="mx-auto flex w-full max-w-180 flex-1 flex-col gap-8 px-4 pt-10 pb-[calc(env(safe-area-inset-bottom)+6rem)] sm:px-6 sm:pb-16"
>
	{#if !$user}
		{#if !$loading}
			<div class="flex flex-col items-center gap-3 py-20 text-center">
				<p class="text-sm text-muted-foreground">Log in to see your settings.</p>
				<Button variant="outline" onclick={login}><LogIn />Log in</Button>
			</div>
		{/if}
	{:else}
		<div class="flex min-w-0 flex-col gap-1">
			<h1 class="text-lg font-semibold tracking-tight">Settings</h1>
			<div class="flex min-w-0 items-center gap-2 text-[13px] text-muted-foreground">
				<span class="truncate">
					<span class="font-medium text-foreground">{nameof($user.email)}</span>
					· {$user.email}
				</span>
				{#if $user.role === 'admin'}
					<span class="shrink-0 rounded-md bg-muted px-1.5 py-0.5 text-[10px]">admin</span>
				{/if}
			</div>
		</div>

		{#if p}
			<section class="flex flex-col">
				<h2 class="pb-1 text-xs font-medium text-muted-foreground">Preferences</h2>
				<div class="flex flex-col divide-y border-y">
					{@render toggle(
						'autoterm',
						'Auto-Archive',
						'Courses you add will get auto-archived once the semester ends. With this off you archive them yourself.',
						p.autoterm,
						setautoterm
					)}
					{@render toggle(
						'examfocus',
						'Upcoming exam feed',
						`My courses shows the papers for the next exam.${
							p.examfocus && next ? ` Now: ${next.label}, until ${shortdate(next.until)}.` : ''
						}`,
						p.examfocus,
						(v) => setsettings({ examfocus: v })
					)}
					{@render toggle(
						'ug1',
						'UG1',
						p.ug1 && p.ug1since
							? `Uses the UG1 exam dates where they differ. Turns off by itself in ${month(ug1until(new Date(p.ug1since)))}.`
							: 'Uses the UG1 exam dates where they differ, like the later first quiz in monsoon. Turns off by itself next May.',
						p.ug1,
						(v) => setsettings({ ug1: v })
					)}
				</div>
			</section>

			<section class="flex flex-col gap-3">
				<h2 class="text-xs font-medium text-muted-foreground">Appearance</h2>
				<ThemePicker large />
			</section>
		{/if}
	{/if}
</main>
