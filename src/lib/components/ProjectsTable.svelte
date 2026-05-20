<script lang="ts">
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';
  import { deleteProject as ipcDelete, removeRecentProject } from '$lib/ipc/projects';
  import { openAndInstall } from '$lib/stores/currentProject';
  import { formatDateSync, type TimezoneMode } from '$lib/helpers/formatDate';
  import ConfirmDeleteProject from './ConfirmDeleteProject.svelte';
  import ConfirmRemoveStaleRecent from './ConfirmRemoveStaleRecent.svelte';
  import type { RecentProjectListEntry } from '$lib/generated/RecentProjectListEntry';

  type Props = {
    projects: RecentProjectListEntry[];
    tzMode: TimezoneMode;
    onchange?: () => void;
  };
  let { projects, tzMode, onchange }: Props = $props();

  let filter = $state('');
  let confirmOpen = $state(false);
  let toDelete = $state<RecentProjectListEntry | null>(null);
  let stalePromptOpen = $state(false);
  let staleEntry = $state<{ path: string; name: string; reason: string } | null>(null);

  let filtered = $derived(
    filter.trim() === ''
      ? projects
      : projects.filter((p) => {
          const q = filter.toLowerCase();
          return p.name.toLowerCase().includes(q) || p.path.toLowerCase().includes(q);
        })
  );

  async function handleOpen(p: RecentProjectListEntry) {
    const result = await openAndInstall(p.path);
    if (result.kind === 'Loaded') {
      goto('/projects/current');
    } else if (result.kind === 'SchemaTooNew') {
      alert(`'${p.name}' requires app v${result.app_version}+; you have v${result.project_version}.`);
    } else if (result.kind === 'Missing') {
      staleEntry = { path: result.path, name: result.name, reason: result.reason };
      stalePromptOpen = true;
    } else if (result.kind === 'Failed') {
      // Sticky-restore branch — not expected here, but be defensive.
      alert(`Failed to open '${p.name}': ${result.reason}`);
      onchange?.();
    }
  }

  async function confirmRemoveStale() {
    if (!staleEntry) return;
    try {
      await removeRecentProject(staleEntry.path);
      onchange?.();
    } finally {
      staleEntry = null;
      stalePromptOpen = false;
    }
  }

  async function requestDelete(p: RecentProjectListEntry) {
    if (!p.folder_exists) {
      // Folder is gone; skip the scary confirm dialog and just remove
      // the recents row. ipcDelete tolerates missing folders.
      try {
        await ipcDelete(p.path);
      } catch (e) {
        alert(`Remove failed: ${JSON.stringify(e)}`);
        return;
      }
      onchange?.();
      return;
    }
    toDelete = p;
    confirmOpen = true;
  }

  async function confirmDelete() {
    if (!toDelete) return;
    try {
      await ipcDelete(toDelete.path);
      onchange?.();
    } catch (e) {
      alert(`Delete failed: ${JSON.stringify(e)}`);
    } finally {
      toDelete = null;
      confirmOpen = false;
    }
  }
</script>

<div class="space-y-4">
  <Input bind:value={filter} placeholder="Filter projects..." class="max-w-sm" />

  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>Name</TableHead>
        <TableHead>Path</TableHead>
        <TableHead>Last opened</TableHead>
        <TableHead>Last modified</TableHead>
        <TableHead class="text-right">Actions</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      {#each filtered as p}
        <TableRow>
          <TableCell class="font-medium">{p.name}</TableCell>
          <TableCell class="max-w-xs break-all text-xs text-slate-600">
            <code>{p.path}</code>
          </TableCell>
          <TableCell>{formatDateSync(p.last_opened_at, tzMode)}</TableCell>
          <TableCell>
            {p.last_modified ? formatDateSync(p.last_modified, tzMode) : '—'}
          </TableCell>
          <TableCell class="space-x-2 text-right">
            <Button size="sm" onclick={() => handleOpen(p)}>Open</Button>
            <Button
              size="sm"
              variant="outline"
              class="text-red-600 hover:bg-red-50"
              onclick={() => requestDelete(p)}
            >
              Delete
            </Button>
          </TableCell>
        </TableRow>
      {/each}
    </TableBody>
  </Table>
</div>

{#if toDelete}
  <ConfirmDeleteProject
    bind:open={confirmOpen}
    name={toDelete.name}
    path={toDelete.path}
    onconfirm={confirmDelete}
  />
{/if}

{#if staleEntry}
  <ConfirmRemoveStaleRecent
    bind:open={stalePromptOpen}
    name={staleEntry.name}
    path={staleEntry.path}
    reason={staleEntry.reason}
    onconfirm={confirmRemoveStale}
  />
{/if}
