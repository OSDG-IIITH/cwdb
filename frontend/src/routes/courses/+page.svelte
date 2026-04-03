<script lang="ts">
    import { listcourses, type Course } from '$lib/api';
    import { onDestroy, onMount } from 'svelte';
    import { Loader2, Search } from '@lucide/svelte';
    import PageHeader from '$lib/components/page-header.svelte';
    import CourseCard from '$lib/components/courses/course-card.svelte';
    import { Input } from '$lib/components/ui/input';
    import * as Pagination from '$lib/components/ui/pagination';

    let courses = $state<Course[]>([]);
    let pinned = $state<Course[]>([]);
    let total = $state(0);
    let currentpage = $state(1);
    let loading = $state(true);
    let showloadingindicator = $state(false);
    let searchquery = $state('');
    let searchinput = $state<HTMLInputElement | null>(null);

    const PER_PAGE = 30;
    const LOADING_DELAY_MS = 250;
    let loadingtimer: ReturnType<typeof setTimeout> | null = null;
    let debounce: ReturnType<typeof setTimeout> | null = null;

    function handlekeydown(e: KeyboardEvent) {
        const activetag = document.activeElement?.tagName;
        if (e.key === '/' && activetag !== 'INPUT' && activetag !== 'TEXTAREA') {
            e.preventDefault();
            searchinput?.focus();
        }
    }

    async function loadcourses() {
        startloading();
        try {
            const res = await listcourses({
                page: currentpage,
                per_page: PER_PAGE,
                q: searchquery || undefined,
            });
            courses = res.courses;
            pinned = res.pinned;
            total = res.total;
        } finally {
            stoploading();
        }
    }

    function startloading() {
        loading = true;
        showloadingindicator = false;
        if (loadingtimer) clearTimeout(loadingtimer);
        loadingtimer = setTimeout(() => {
            if (loading) showloadingindicator = true;
        }, LOADING_DELAY_MS);
    }

    function stoploading() {
        loading = false;
        showloadingindicator = false;
        if (loadingtimer) {
            clearTimeout(loadingtimer);
            loadingtimer = null;
        }
    }

    function handlesearch() {
        if (debounce) clearTimeout(debounce);
        debounce = setTimeout(() => {
            currentpage = 1;
            loadcourses();
        }, 200);
    }

    function handlepageturn() {
        loadcourses();
    }

    onDestroy(() => {
        if (loadingtimer) clearTimeout(loadingtimer);
        if (debounce) clearTimeout(debounce);
    });

    onMount(() => {
        loadcourses();
    });

    let totalpages = $derived(Math.ceil(total / PER_PAGE));
    let empty = $derived(!loading && courses.length === 0 && pinned.length === 0);
</script>

<svelte:window onkeydown={handlekeydown} />

<div class="min-h-screen bg-background text-foreground flex flex-col font-sans">
    <PageHeader title="Courses" />

    <main class="flex-1 container mx-auto p-6 max-w-6xl">
        <div class="flex items-center justify-between gap-4 mb-8">
            <div class="relative w-full md:w-64">
                <Search class="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input
                    placeholder="search courses"
                    class="pl-8 focus-visible:ring-0"
                    bind:value={searchquery}
                    bind:ref={searchinput}
                    oninput={handlesearch}
                />
            </div>

            <p class="text-sm text-muted-foreground whitespace-nowrap">
                {total + pinned.length} {total + pinned.length === 1 ? 'course' : 'courses'}
            </p>
        </div>

        {#if loading && showloadingindicator}
            <div class="flex justify-center items-center py-20">
                <Loader2 class="w-8 h-8 animate-spin text-muted-foreground" />
            </div>
        {:else if loading}
            <div class="py-20" aria-hidden="true"></div>
        {:else if empty && !searchquery}
            <div class="text-center py-20">
                <p class="text-muted-foreground">no courses found.</p>
            </div>
        {:else if empty}
            <div class="text-center py-20">
                <p class="text-muted-foreground">no matching courses found.</p>
            </div>
        {:else}
            {#if pinned.length > 0}
                <div class="mb-8">
                    <p class="text-xs font-medium text-muted-foreground mb-3 uppercase tracking-wider">Pinned</p>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                        {#each pinned as course (course.id)}
                            <CourseCard {course} onpintoggle={loadcourses} />
                        {/each}
                    </div>
                </div>
            {/if}

            {#if courses.length > 0}
                {#if pinned.length > 0}
                    <p class="text-xs font-medium text-muted-foreground mb-3 uppercase tracking-wider">All Courses</p>
                {/if}
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                    {#each courses as course (course.id)}
                        <CourseCard {course} onpintoggle={loadcourses} />
                    {/each}
                </div>
            {/if}

            {#if totalpages > 1}
                <div class="mt-8">
                    <Pagination.Root count={total} perPage={PER_PAGE} bind:page={currentpage} onPageChange={handlepageturn}>
                        {#snippet children({ pages })}
                            <Pagination.Content>
                                <Pagination.Item>
                                    <Pagination.PrevButton />
                                </Pagination.Item>
                                {#each pages as p (p.key)}
                                    {#if p.type === 'ellipsis'}
                                        <Pagination.Item>
                                            <Pagination.Ellipsis />
                                        </Pagination.Item>
                                    {:else}
                                        <Pagination.Item>
                                            <Pagination.Link page={p} isActive={currentpage === p.value}>
                                                {p.value}
                                            </Pagination.Link>
                                        </Pagination.Item>
                                    {/if}
                                {/each}
                                <Pagination.Item>
                                    <Pagination.NextButton />
                                </Pagination.Item>
                            </Pagination.Content>
                        {/snippet}
                    </Pagination.Root>
                </div>
            {/if}
        {/if}
    </main>
</div>
