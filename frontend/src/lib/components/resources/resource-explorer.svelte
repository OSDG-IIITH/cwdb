<script lang="ts">
    import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui/table';
    import * as Breadcrumb from '$lib/components/ui/breadcrumb';
    import { Folder, FileText, Home } from '@lucide/svelte';
    import type { Resource, Source } from '$lib/api';

    import { goto } from '$app/navigation';

    let { resources, sources = [], path = [] } = $props<{ resources: Resource[], sources?: Source[], path?: string[] }>();

    let currentPath = $derived.by(() => {
        if (path.length < 2) return [];
        const [owner, repo, ...rest] = path;
        return [`${owner}/${repo}`, ...rest];
    });

    type FSNode = {
        name: string;
        path: string[];
        type: 'repo' | 'folder' | 'file';
        children: Map<string, FSNode>;
        resource?: Resource;
    };

    let root = $derived.by(() => {
        const rootNode: FSNode = { name: 'root', path: [], type: 'folder', children: new Map() };
        
        if (path.length < 2) {
            for (const s of sources) {
                const repoName = `${s.owner}/${s.repo}`;
                if (!rootNode.children.has(repoName)) {
                    rootNode.children.set(repoName, { name: repoName, path: [s.owner, s.repo], type: 'repo', children: new Map() });
                }
            }
            return rootNode;
        }

        for (const res of resources) {
            const repoName = `${res.owner}/${res.repo}`;
            let repoNode = rootNode.children.get(repoName);
            if (!repoNode) {
                repoNode = { name: repoName, path: [res.owner, res.repo], type: 'repo', children: new Map() };
                rootNode.children.set(repoName, repoNode);
            }

            const parts = res.file_path.split('/');
            let currentNode = repoNode;
            
            for (let i = 0; i < parts.length; i++) {
                const part = parts[i];
                const isLast = i === parts.length - 1;
                
                if (isLast) {
                    currentNode.children.set(part, {
                        name: part,
                        path: [...repoNode.path, ...parts.slice(0, i + 1)],
                        type: 'file',
                        children: new Map(),
                        resource: res
                    });
                } else {
                    let folder = currentNode.children.get(part);
                    if (!folder) {
                        folder = {
                            name: part,
                            path: [...repoNode.path, ...parts.slice(0, i + 1)],
                            type: 'folder',
                            children: new Map()
                        };
                        currentNode.children.set(part, folder);
                    }
                    currentNode = folder;
                }
            }
        }
        return rootNode;
    });

    let currentView = $derived.by(() => {
        let node = root;
        for (const part of currentPath) {
            node = node?.children.get(part) as FSNode;
            if (!node) return [];
        }

        const items = Array.from(node.children.values());
        return items.sort((a, b) => {
            if (a.type !== b.type) {
                if (a.type === 'repo') return -1;
                if (b.type === 'repo') return 1;
                if (a.type === 'folder') return -1;
                return 1;
            }
            return a.name.localeCompare(b.name);
        });
    });

    function navigate(p: string[]) {
        goto(`/resources/${p.join('/')}`);
    }

    function navigateUp() {
        if (path.length === 0) return;
        const newPath = path.slice(0, -1);
        if (newPath.length < 2) {
            goto('/resources');
        } else {
            goto(`/resources/${newPath.join('/')}`);
        }
    }
    function rawUrl(r: Resource) {
        const b = r.branch || 'main';
        return `https://raw.githubusercontent.com/${r.owner}/${r.repo}/${b}/${encodeURI(r.file_path)}`;
    }
</script>

<div class="space-y-3">
    <div class="flex flex-col gap-4">
        <Breadcrumb.Root>
            <Breadcrumb.List>
                <Breadcrumb.Item>
                    <Breadcrumb.Link href="/resources" class="font-mono font-bold hover:bg-muted/50 px-2 py-1 rounded-md transition-colors block">
                        cwdb
                    </Breadcrumb.Link>
                </Breadcrumb.Item>
                {#each currentPath as part, i}
                    <Breadcrumb.Separator />
                    <Breadcrumb.Item>
                        <Breadcrumb.Link href={`/resources/${currentPath.slice(0, i + 1).join('/')}`} class="hover:bg-muted/50 px-2 py-1 rounded-md transition-colors block">
                            {part}
                        </Breadcrumb.Link>
                    </Breadcrumb.Item>
                {/each}
            </Breadcrumb.List>
        </Breadcrumb.Root>
    </div>

    <div class="rounded-xl border border-border/60 bg-card/40 shadow-sm backdrop-blur-sm overflow-hidden">

        <div class="p-0">
            <Table>
                <TableHeader class="hover:bg-transparent">
                    <TableRow class="hover:bg-transparent border-border/40">
                        <TableHead class="w-[50px] pl-6"></TableHead>
                        <TableHead>Name</TableHead>
                        <TableHead class="w-[100px] text-right pr-6">Kind</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {#if currentPath.length > 0}
                        <TableRow class="hover:bg-muted/50 dark:hover:bg-accent/40 cursor-pointer border-border/40 group transition-colors" onclick={navigateUp}>
                            <TableCell class="pl-6 py-3"><span class="text-xl text-muted-foreground group-hover:text-foreground transition-colors">..</span></TableCell>
                            <TableCell class="font-medium text-muted-foreground group-hover:text-foreground py-3">..</TableCell>
                            <TableCell class="pr-6 py-3"></TableCell>
                        </TableRow>
                    {/if}

                    {#each currentView as item}
                        <TableRow 
                            class="hover:bg-muted/50 dark:hover:bg-accent/40 cursor-pointer border-border/40 group transition-colors" 
                            onclick={() => {
                                if (item.type === 'file' && item.resource) {
                                    window.open(rawUrl(item.resource), '_blank');
                                } else {
                                    navigate(item.path);
                                }
                            }}
                        >
                            <TableCell class="pl-6 py-3">
                                {#if item.type === 'repo'}
                                    <Folder class="h-5 w-5 text-primary fill-primary/20 transition-colors" /> 
                                {:else if item.type === 'folder'}
                                    <Folder class="h-5 w-5 text-blue-400 fill-blue-400/20 transition-colors" />
                                {:else}
                                    <FileText class="h-5 w-5 text-muted-foreground fill-foreground/5 group-hover:text-foreground transition-colors" />
                                {/if}
                            </TableCell>
                                <TableCell class="font-medium py-3 text-foreground/90 group-hover:text-foreground break-all">
                                    {item.name}
                                </TableCell>
                            <TableCell class="text-right text-[10px] uppercase tracking-wider font-semibold text-muted-foreground pr-6 py-3">
                                {item.type}
                            </TableCell>
                        </TableRow>
                    {:else}
                        <TableRow class="hover:bg-transparent border-transparent">
                            <TableCell colspan={3} class="h-32 text-center text-muted-foreground">
                                <div class="flex flex-col items-center justify-center gap-2">
                                    <Folder class="h-8 w-8 text-muted-foreground/20" />
                                    <p>No files found.</p>
                                </div>
                            </TableCell>
                        </TableRow>
                    {/each}
                </TableBody>
            </Table>
        </div>
    </div>
</div>
