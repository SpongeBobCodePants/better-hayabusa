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
  import { deleteProject as ipcDelete } from '$lib/ipc/projects';
  import { openAndInstall } from '$lib/stores/currentProject';
  import { formatDateSync, type TimezoneMode } from '$lib/helpers/formatDate';
  import ConfirmDeleteProject from './ConfirmDeleteProject.svelte';
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
    } else if (result.kind === 'Failed') {
      alert(`Failed to open '${p.name}': ${result.reason}`);
      onchange?.();
    }
  }

  function requestDelete(p: RecentProjectListEntry) {
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
        <TableHead>Last opened</TableHead>
        <TableHead>Last modified</TableHead>
        <TableHead class="text-right">Actions</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      {#each filtered as p}
        <TableRow>
          <TableCell class="font-medium">{p.name}</TableCell>
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
