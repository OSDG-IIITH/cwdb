<script lang="ts">
	import { listSources, publish, type Source } from '$lib/api';
	import { onDestroy, onMount } from 'svelte';
	import SourceCard from '$lib/components/sources/source-card.svelte';
	import NewSourceDialog from '$lib/components/sources/new-source-dialog.svelte';
	import { Loader2 } from '@lucide/svelte';

	import PageHeader from '$lib/components/page-header.svelte';
	import { Input } from '$lib/components/ui/input';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { Button } from '$lib/components/ui/button';
	import { Search, ArrowUpDown, Calendar, ArrowDownAZ, ArrowUpAZ } from '@lucide/svelte';
	import { makeloadingstate } from '$lib/hooks/loading';
	import { bindslashfocus } from '$lib/hooks/shortcuts';

	let sources = $state<Source[]>([]);
	let loading = $state(true);
	let showLoadingIndicator = $state(false);
	let searchQuery = $state('');
	let sortBy = $state<'date-desc' | 'date-asc' | 'name-asc' | 'name-desc'>('date-desc');
	let searchInput = $state<HTMLInputElement | null>(null);

	let loadingcontrol = makeloadingstate(
		(value) => {
			loading = value;
		},
		(value) => {
			showLoadingIndicator = value;
		}
	);
	let unbindslash: (() => void) | null = null;
	let publishing = $state(false);

	async function handlePublish() {
		publishing = true;
		try {
			const result = await publish();
			if (result) {
				console.log('Published:', result);
			}
		} finally {
			publishing = false;
		}
	}

	let filteredSources = $derived(
		sources
			.filter((s) => {
				const query = searchQuery.toLowerCase();
				return s.owner.toLowerCase().includes(query) || s.repo.toLowerCase().includes(query);
			})
			.sort((a, b) => {
				// TODO: switch to created_at?
				if (sortBy === 'date-desc') return (b.id || 0) - (a.id || 0);
				if (sortBy === 'date-asc') return (a.id || 0) - (b.id || 0);
				if (sortBy === 'name-asc')
					return `${a.owner}/${a.repo}`.localeCompare(`${b.owner}/${b.repo}`);
				if (sortBy === 'name-desc')
					return `${b.owner}/${b.repo}`.localeCompare(`${a.owner}/${a.repo}`);
				return 0;
			})
	);

	async function loadSources() {
		loadingcontrol.start();
		try {
			sources = await listSources();
		} finally {
			loadingcontrol.stop();
		}
	}

	onDestroy(() => {
		loadingcontrol.destroy();
		unbindslash?.();
	});

	onMount(() => {
		unbindslash = bindslashfocus(() => searchInput);
		loadSources();
	});
</script>

<div class="flex min-h-screen flex-col bg-background font-sans text-foreground">
	<PageHeader title="Sources" />

	<main class="container mx-auto max-w-6xl flex-1 p-6">
		<div class="mb-8 flex flex-col justify-between gap-4 md:flex-row md:items-center">
			<div class="flex w-full items-center gap-2 md:w-auto">
				<div class="relative w-full md:w-64">
					<Search class="absolute top-2.5 left-2 h-4 w-4 text-muted-foreground" />
					<Input
						placeholder="search"
						class="pl-8 focus-visible:ring-0"
						bind:value={searchQuery}
						bind:ref={searchInput}
					/>
				</div>

				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						{#snippet child({ props })}
							<Button variant="outline" size="icon" {...props}>
								<ArrowUpDown class="h-4 w-4" />
							</Button>
						{/snippet}
					</DropdownMenu.Trigger>
					<DropdownMenu.Content align="end">
						<DropdownMenu.Label>Sort by</DropdownMenu.Label>
						<DropdownMenu.Separator />
						<DropdownMenu.Item onclick={() => (sortBy = 'date-desc')}>
							<Calendar class="mr-2 h-4 w-4" />
							Newest First
							{#if sortBy === 'date-desc'}
								<span class="ml-auto text-xs">✓</span>
							{/if}
						</DropdownMenu.Item>
						<DropdownMenu.Item onclick={() => (sortBy = 'date-asc')}>
							<Calendar class="mr-2 h-4 w-4" />
							Oldest First
							{#if sortBy === 'date-asc'}
								<span class="ml-auto text-xs">✓</span>
							{/if}
						</DropdownMenu.Item>
						<DropdownMenu.Item onclick={() => (sortBy = 'name-asc')}>
							<ArrowDownAZ class="mr-2 h-4 w-4" />
							Name (A-Z)
							{#if sortBy === 'name-asc'}
								<span class="ml-auto text-xs">✓</span>
							{/if}
						</DropdownMenu.Item>
						<DropdownMenu.Item onclick={() => (sortBy = 'name-desc')}>
							<ArrowUpAZ class="mr-2 h-4 w-4" />
							Name (Z-A)
							{#if sortBy === 'name-desc'}
								<span class="ml-auto text-xs">✓</span>
							{/if}
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			</div>

			<div class="flex items-center gap-2">
				<Button variant="outline" onclick={handlePublish} disabled={publishing}>
					{#if publishing}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
					{/if}
					Publish
				</Button>
				<NewSourceDialog on:created={loadSources} />
			</div>
		</div>

		{#if loading && showLoadingIndicator}
			<div class="flex items-center justify-center py-20">
				<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
			</div>
		{:else if loading}
			<div class="py-20" aria-hidden="true"></div>
		{:else if sources.length === 0 && searchQuery === ''}
			<div class="py-20 text-center">
				<p class="mb-4 text-muted-foreground">No sources found.</p>
			</div>
		{:else if filteredSources.length === 0}
			<div class="py-20 text-center">
				<p class="mb-4 text-muted-foreground">No matching sources found.</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
				{#each filteredSources as source (source.id)}
					<SourceCard {source} onrefresh={loadSources} />
				{/each}
			</div>
		{/if}
	</main>
</div>
