<script lang="ts">
    import { createSource } from '$lib/api';
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
    import { Label } from '$lib/components/ui/label';
    import * as Dialog from '$lib/components/ui/dialog';
    import { createEventDispatcher } from 'svelte';
    import { Plus } from '@lucide/svelte';

    let open = $state(false);
    let loading = $state(false);
    let owner = $state('');
    let repo = $state('');
    let branch = $state('');
    let error = $state('');

    const dispatch = createEventDispatcher();

    async function handleSubmit() {
        if (!owner || !repo) {
            error = "Owner and Repo are required";
            return;
        }
        loading = true;
        error = '';
        const res = await createSource(owner, repo, branch || undefined);
        loading = false;

        if (res) {
            open = false;
            owner = '';
            repo = '';
            branch = '';
            dispatch('created');
        } else {
            error = "Failed to create source. Check if it exists or access is valid.";
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Trigger>
        {#snippet child({ props })}
            <Button {...props}>
                <Plus class="w-4 h-4 mr-2" />
                Add Source
            </Button>
        {/snippet}
    </Dialog.Trigger>
    <Dialog.Content class="sm:max-w-[425px]">
        <Dialog.Header>
            <Dialog.Title>Add New Source</Dialog.Title>
            <Dialog.Description>
                Connect a GitHub repository to index its contents.
            </Dialog.Description>
        </Dialog.Header>
        <div class="grid gap-4 py-4">
            <div class="grid grid-cols-4 items-center gap-4">
                <Label for="owner" class="text-right">Owner</Label>
                <Input id="owner" bind:value={owner} placeholder="facebook" class="col-span-3" />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <Label for="repo" class="text-right">Repo</Label>
                <Input id="repo" bind:value={repo} placeholder="react" class="col-span-3" />
            </div>
             <div class="grid grid-cols-4 items-center gap-4">
                <Label for="branch" class="text-right">Branch</Label>
                <Input id="branch" bind:value={branch} placeholder="(optional)" class="col-span-3" />
            </div>
            {#if error}
                <div class="text-sm text-red-500 font-medium">
                    {error}
                </div>
            {/if}
        </div>
        <Dialog.Footer>
            <Button type="submit" onclick={handleSubmit} disabled={loading}>
                {#if loading}
                    Adding...
                {:else}
                    Add Source
                {/if}
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
