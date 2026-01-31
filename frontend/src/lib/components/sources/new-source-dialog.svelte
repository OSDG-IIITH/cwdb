<script lang="ts">
    import { createSource, refreshSource } from '$lib/api';
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
    import { Label } from '$lib/components/ui/label';
    import * as Dialog from '$lib/components/ui/dialog';
    import { createEventDispatcher } from 'svelte';
    import { Plus, Github, Loader2, Check, AlertCircle, GitBranch, Star } from '@lucide/svelte';
    import { cn } from '$lib/utils';
    import { user } from '$lib/auth';
    import { toast } from 'svelte-sonner';

    let open = $state(false);
    let loading = $state(false);
    
    // State for URL parsing and verification
    let url = $state('');
    let verifying = $state(false);
    let verified = $state(false);
    let error = $state('');
    
    let parsedData = $state({ owner: '', repo: '', branch: '' });
    let repoDetails = $state<{ description: string; stargazers_count: number; default_branch: string } | null>(null);

    const dispatch = createEventDispatcher();

    let timer: ReturnType<typeof setTimeout>;

    function handleInput(e: Event) {
        const input = (e.target as HTMLInputElement).value;
        url = input;
        
        verified = false;
        error = '';
        repoDetails = null;
        parsedData = { owner: '', repo: '', branch: '' };

        clearTimeout(timer);
        
        if (!input.trim()) return;

        timer = setTimeout(() => {
            parseAndVerify(input);
        }, 800);
    }

    async function parseAndVerify(inputUrl: string) {
        try {
            // supports:
            // https://github.com/owner/repo
            // https://github.com/owner/repo/tree/branch
            // owner/repo
            
            let cleanUrl = inputUrl.trim();
            
            if (!cleanUrl.startsWith('http') && !cleanUrl.includes('github.com') && cleanUrl.split('/').length === 2) {
                const [o, r] = cleanUrl.split('/');
                parsedData = { owner: o, repo: r, branch: '' };
            } else {
                 try {
                    const u = new URL(cleanUrl);
                    if (u.hostname !== 'github.com') {
                        error = "Not a GitHub URL";
                        return;
                    }
                    const parts = u.pathname.split('/').filter(Boolean);
                    if (parts.length < 2) {
                        error = "Invalid repository URL";
                        return;
                    }
                    parsedData.owner = parts[0];
                    parsedData.repo = parts[1];
                    
                    if (parts[2] === 'tree' && parts[3]) {
                        parsedData.branch = parts[3];
                    }
                } catch {
                     error = "Invalid URL format";
                     return;
                }
            }

            if (!parsedData.owner || !parsedData.repo) return;

            verifying = true;
            
            // Verify with GitHub API
            const res = await fetch(`https://api.github.com/repos/${parsedData.owner}/${parsedData.repo}`);
            
            if (res.ok) {
                const data = await res.json();
                repoDetails = {
                    description: data.description,
                    stargazers_count: data.stargazers_count,
                    default_branch: data.default_branch
                };
                verified = true;
            } else {
                if (res.status === 404) error = "Repository not found or private";
                else error = "Failed to verify repository";
            }
        } catch (e) {
            error = "Error verifying repository";
        } finally {
            verifying = false;
        }
    }

    async function handleSubmit() {
        if (!verified || !parsedData.owner || !parsedData.repo) return;
        
        loading = true;
        const branchToUse = parsedData.branch || repoDetails?.default_branch;
        const res = await createSource(parsedData.owner, parsedData.repo, branchToUse);

        if (res) {
            await refreshSource(res.id);
        }
        
        loading = false;

        if (res) {
            open = false;
            // Reset everything
            url = '';
            parsedData = { owner: '', repo: '', branch: '' };
            repoDetails = null;
            verified = false;
            dispatch('created');
        } else {
            error = "Failed to add source to backend";
        }
    }
</script>

<Button onclick={() => {
    if (!$user) {
        toast.error("Authentication Required", {
            description: "You must be logged in to add a source."
        });
        return;
    }
    open = true;
}}>
    <Plus class="w-4 h-4 mr-2" />
    Add Source
</Button>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[500px] border-border/60 bg-background/95 backdrop-blur-xl">
        <Dialog.Header>
            <Dialog.Title class="text-xl">Add New Source</Dialog.Title>
            <Dialog.Description>
                Enter a GitHub repository URL to index.
            </Dialog.Description>
        </Dialog.Header>

        <div class="grid gap-6 py-4">
            <div class="space-y-2">
                <Label for="url">Repository URL</Label>
                <div class="relative">
                    <Input 
                        id="url" 
                        value={url} 
                        oninput={handleInput}
                        placeholder="https://github.com/owner/repo" 
                        class="pr-10 font-mono text-sm focus-visible:ring-0"
                    />
                    <div class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground">
                        {#if verifying}
                            <Loader2 class="h-4 w-4 animate-spin" />
                        {:else if verified}
                            <Check class="h-4 w-4 text-green-500" />
                        {:else if error}
                            <AlertCircle class="h-4 w-4 text-destructive" />
                        {/if}
                    </div>
                </div>
                {#if error}
                    <p class="text-xs text-destructive mt-1.5 px-1">{error}</p>
                {/if}
            </div>

            {#if verified && repoDetails}
                <div class="rounded-lg border border-border/60 bg-muted/30 p-4 transition-all animate-in fade-in slide-in-from-top-2">
                    <div class="flex items-start gap-4">
                        <div class="rounded-md bg-background p-2 ring-1 ring-border">
                            <Github class="h-6 w-6" />
                        </div>
                        <div class="space-y-1 overflow-hidden">
                            <h4 class="font-medium leading-none tracking-tight flex items-center gap-2">
                                {parsedData.owner}/{parsedData.repo}
                            </h4>
                            <p class="text-xs text-muted-foreground line-clamp-2">
                                {repoDetails.description || 'No description provided.'}
                            </p>
                            
                            <div class="flex items-center gap-4 mt-3 text-xs text-muted-foreground">
                                <span class="flex items-center gap-1">
                                    <Star class="h-3 w-3" />
                                    {repoDetails.stargazers_count.toLocaleString()}
                                </span>
                                <span class="flex items-center gap-1">
                                    <GitBranch class="h-3 w-3" />
                                    {parsedData.branch || repoDetails.default_branch}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            {/if}
        </div>

        <Dialog.Footer>
            <Button 
                type="submit" 
                onclick={handleSubmit} 
                disabled={!verified || loading || verifying}
                class="w-full sm:w-auto"
            >
                {#if loading}
                    <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                    Adding Source...
                {:else}
                    Add Repository
                {/if}
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
