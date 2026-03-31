<script lang="ts">
	import { onMount } from 'svelte';
	import { searchResources, type Resource } from '$lib/api';
	import { Input } from '$lib/components/ui/input';

	let query = $state('');
	let results = $state<Resource[]>([]);

	async function search() {
		results = await searchResources(query);
	}

	onMount(search);

	function rawUrl(r: Resource) {
		const b = r.branch || 'main';
		return `https://raw.githubusercontent.com/${r.owner}/${r.repo}/${b}/${encodeURI(r.file_path)}`;
	}

	function repoUrl(r: Resource) {
		return `https://github.com/${r.owner}/${r.repo}`;
	}

	function openRaw(r: Resource) {
		window.open(rawUrl(r), '_blank');
	}

	let searchInput: HTMLInputElement | null = $state(null);

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === '/' && document.activeElement !== searchInput) {
			e.preventDefault();
			searchInput?.focus();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="max-w-3xl mx-auto px-4 min-h-screen flex flex-col justify-between relative">
	<!-- logo + searchbar -->
	<div
		class={query.length > 0
			? 'fixed top-0 left-0 right-0 z-40 bg-background border-b border-border'
			: 'flex-1 flex flex-col justify-center items-center text-center'}
	>
		<div
			class={`w-full ${
				query.length > 0
					? 'max-w-3xl mx-auto px-4 py-4 flex flex-col md:flex-row md:items-center md:justify-between gap-4'
					: 'flex flex-col items-center gap-4'
			}`}
		>
			<!-- logo -->
			<div
				class={`font-bold text-foreground font-mono tracking-tight ${
					query.length > 0 ? 'text-3xl' : 'text-8xl'
				}`}
			>
				cwdb
			</div>

			<!-- searchbar -->
			<div class={`relative w-full ${query.length === 0 ? 'mt-2' : 'mt-0'}`}>
				<svg
					class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground pointer-events-none z-10"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					viewBox="0 0 24 24"
					aria-hidden="true"
				>
					<circle cx="11" cy="11" r="7" stroke="currentColor" />
					<line x1="16.65" y1="16.65" x2="21" y2="21" stroke="currentColor" />
				</svg>
				<Input
					bind:ref={searchInput}
					type="text"
					class="w-full pl-10 pr-4 py-3 h-auto border-border focus-visible:ring-0 focus-visible:border-border"
					placeholder="search"
					bind:value={query}
					oninput={search}
				/>
			</div>
		</div>
	</div>

	<!-- results -->
	{#if query.length > 0}
		<div class="flex-1 mt-32 sm:mt-20">
			<ul class="mt-4 mb-10 space-y-4">
				{#if results.length > 0}
					{#each results as r (r.id)}
						<li>
							<button
								type="button"
								class="w-full text-left cursor-pointer p-4 border border-border rounded-md bg-background hover:bg-muted/30 transition-colors"
								onclick={() => openRaw(r)}
							>
								<div class="text-base font-medium text-foreground">{r.title}</div>
								<div class="text-xs text-muted-foreground mt-1">{r.file_path}</div>

								<div class="flex flex-wrap items-center gap-2 mt-3 text-xs text-muted-foreground">
									<a
										href={repoUrl(r)}
										target="_blank"
										rel="noopener"
										class="px-2 py-0.5 bg-muted rounded hover:underline"
										onclick={(e: MouseEvent) => e.stopPropagation()}
									>
										{r.owner}/{r.repo}
									</a>

									{#if r.type}
										<span class="px-2 py-0.5 bg-muted rounded">
											{r.type}
										</span>
									{/if}

									{#if r.course_name}
										<span class="px-2 py-0.5 bg-muted rounded">
											{r.course_name.replace(/\b\w/g, (c) => c.toUpperCase())}
										</span>
									{/if}
								</div>
							</button>
						</li>
					{/each}
				{:else}
					<li class="text-center text-sm text-muted-foreground">No results found.</li>
				{/if}
			</ul>
		</div>
	{/if}

	<!-- footer -->
	<footer class="mt-10 hidden py-6 text-center text-sm text-muted-foreground md:block">
		Made with ♥ by
		<a
			href="https://github.com/nuxshed"
			class="underline hover:opacity-80"
			target="_blank"
			rel="noopener"
		>
			Advait Dintakurti
		</a>
	</footer>
</div>
