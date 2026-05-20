<script lang="ts">
  import { goto } from '$app/navigation';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import {
    Sheet,
    SheetContent,
    SheetDescription,
    SheetFooter,
    SheetHeader,
    SheetTitle,
  } from '$lib/components/ui/sheet';
  import { createAndInstall, openAndInstall } from '$lib/stores/currentProject';
  import type { CommandError } from '$lib/generated/CommandError';

  type Props = {
    open: boolean;
    oncreate?: () => void;
  };
  let { open = $bindable(false), oncreate }: Props = $props();

  let name = $state('');
  let folder = $state('');
  let description = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);
  let conflictPath = $state<string | null>(null);

  function reset() {
    name = '';
    folder = '';
    description = '';
    error = null;
    conflictPath = null;
  }

  async function pickFolder() {
    const picked = await openDialog({ directory: true, multiple: false });
    if (typeof picked === 'string') {
      folder = picked;
      conflictPath = null;
      error = null;
    }
  }

  async function handleCreate() {
    if (!name.trim() || !folder.trim()) {
      error = 'Name and folder are required.';
      return;
    }
    busy = true;
    error = null;
    conflictPath = null;
    try {
      await createAndInstall(folder, name.trim(), description.trim() || undefined);
      oncreate?.();
      open = false;
      reset();
      goto('/projects/current');
    } catch (e) {
      const err = e as CommandError;
      if (err.kind === 'AlreadyExists') {
        conflictPath = err.path;
      } else if (err.kind === 'Io') {
        error = `I/O error: ${err.message}`;
      } else if (err.kind === 'Db') {
        error = `Database error: ${err.message}`;
      } else {
        error = `Failed to create project (${err.kind}).`;
      }
    } finally {
      busy = false;
    }
  }

  async function handleOpenInstead() {
    if (!conflictPath) return;
    busy = true;
    try {
      const result = await openAndInstall(conflictPath);
      if (result.kind === 'Loaded') {
        open = false;
        reset();
        goto('/projects/current');
      } else if (result.kind === 'SchemaTooNew') {
        error = `That project requires a newer app version.`;
      }
    } finally {
      busy = false;
    }
  }
</script>

<Sheet bind:open>
  <SheetContent side="right" class="sm:max-w-md">
    <SheetHeader>
      <SheetTitle>New project</SheetTitle>
      <SheetDescription>
        Create a new investigation project. You can change settings later.
      </SheetDescription>
    </SheetHeader>

    <div class="space-y-4 py-4">
      <div>
        <Label for="name">Project name</Label>
        <Input id="name" bind:value={name} placeholder="APT-29 sweep" />
      </div>

      <div>
        <Label for="folder">Default folder</Label>
        <div class="flex gap-2">
          <Input id="folder" bind:value={folder} readonly placeholder="Click Browse to pick a folder" />
          <Button variant="outline" onclick={pickFolder}>Browse</Button>
        </div>
        <p class="mt-1 text-xs text-slate-500">
          Source evidence and output paths can be set per-job to anywhere — this
          is just where the project metadata and logs live, and the default
          starting point for new jobs.
        </p>
      </div>

      <div>
        <Label for="description">Description (optional)</Label>
        <Textarea id="description" bind:value={description} rows={3} />
      </div>

      {#if conflictPath}
        <Alert>
          <AlertDescription>
            This folder is already a project — open it?
            <div class="mt-2 flex gap-2">
              <Button size="sm" onclick={handleOpenInstead}>Open</Button>
              <Button size="sm" variant="outline" onclick={() => (conflictPath = null)}>
                Cancel
              </Button>
            </div>
          </AlertDescription>
        </Alert>
      {/if}

      {#if error}
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      {/if}
    </div>

    <SheetFooter class="flex gap-2">
      <Button variant="outline" onclick={() => { open = false; reset(); }}>Cancel</Button>
      <Button onclick={handleCreate} disabled={busy}>
        {busy ? 'Creating...' : 'Create'}
      </Button>
    </SheetFooter>
  </SheetContent>
</Sheet>
