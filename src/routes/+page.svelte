<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';
  import { listAllProjects, removeRecentProject } from '$lib/ipc/projects';
  import { openAndInstall } from '$lib/stores/currentProject';
  import { getRecentProjectsCount } from '$lib/ipc/app';
  import NewProjectSheet from '$lib/components/NewProjectSheet.svelte';
  import ConfirmRemoveStaleRecent from '$lib/components/ConfirmRemoveStaleRecent.svelte';
  import type { RecentProjectListEntry } from '$lib/generated/RecentProjectListEntry';

  let projects = $state<RecentProjectListEntry[]>([]);
  let count = $state<number>(5);
  let loaded = $state(false);
  let newSheetOpen = $state(false);
  let stalePromptOpen = $state(false);
  let staleEntry = $state<{ path: string; name: string; reason: string } | null>(null);

  // Sort by last_modified desc; rows with null last_modified go to the end.
  let sortedAndLimited = $derived(
    [...projects]
      .sort((a, b) => {
        if (a.last_modified === null && b.last_modified === null) return 0;
        if (a.last_modified === null) return 1;
        if (b.last_modified === null) return -1;
        return b.last_modified.localeCompare(a.last_modified);
      })
      .slice(0, count)
  );

  onMount(async () => {
    [projects, count] = await Promise.all([
      listAllProjects(),
      getRecentProjectsCount(),
    ]);
    loaded = true;
  });

  async function refreshRecents() {
    projects = await listAllProjects();
  }

  async function handleOpenRecent(r: RecentProjectListEntry) {
    const result = await openAndInstall(r.path);
    if (result.kind === 'Loaded') {
      goto('/projects/current');
    } else if (result.kind === 'SchemaTooNew') {
      alert(`Project requires app v${result.app_version}+; you have v${result.project_version}.`);
    } else if (result.kind === 'Missing') {
      staleEntry = { path: result.path, name: result.name, reason: result.reason };
      stalePromptOpen = true;
    } else if (result.kind === 'Failed') {
      alert(`Failed to open '${r.name}': ${result.reason}`);
      refreshRecents();
    }
  }

  async function confirmRemoveStale() {
    if (!staleEntry) return;
    try {
      await removeRecentProject(staleEntry.path);
      refreshRecents();
    } finally {
      staleEntry = null;
      stalePromptOpen = false;
    }
  }
</script>

<div class="mx-auto max-w-3xl space-y-6 p-8">
  <header>
    <h1 class="text-3xl font-bold">Better Hayabusa</h1>
    <p class="mt-1 text-sm italic text-slate-500">
      Making your life suck a little less...
    </p>
    <p class="mt-2 text-slate-600">
      A graphical UI for Hayabusa and related tools (Chainsaw and Takajo).
      Organize your investigations as projects, configure tool runs as named
      jobs, and review run history in one place.
    </p>
  </header>

  <Card>
    <CardHeader>
      <CardTitle>Get started</CardTitle>
    </CardHeader>
    <CardContent class="flex gap-3">
      <Button onclick={() => (newSheetOpen = true)}>New project</Button>
      <Button
        variant="outline"
        disabled={loaded && projects.length === 0}
        onclick={() => goto('/projects/')}
      >
        Open project
      </Button>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Recent projects</CardTitle>
    </CardHeader>
    <CardContent>
      {#if !loaded}
        <p class="text-sm text-slate-500">Loading...</p>
      {:else if projects.length === 0}
        <div class="flex flex-col items-center gap-3 py-6 text-center">
          <img
            src="/img/no-projects-placeholder.svg"
            alt="No projects"
            class="h-32 w-32"
          />
          <p class="text-sm font-medium text-slate-700">
            You have no projects. Looks like somebody better get off their ass
            and GET TO WORK!
          </p>
        </div>
      {:else}
        <Table class="table-fixed">
          <TableHeader>
            <TableRow>
              <TableHead class="w-[40%]">Name</TableHead>
              <TableHead class="w-[60%]">Description</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {#each sortedAndLimited as r}
              <TableRow
                class="cursor-pointer hover:bg-slate-100"
                onclick={() => handleOpenRecent(r)}
              >
                <TableCell class="font-medium">
                  <div class="truncate" title={r.name}>{r.name}</div>
                </TableCell>
                <TableCell class="text-slate-500">
                  {#if r.description}
                    <div class="truncate" title={r.description}>{r.description}</div>
                  {:else}
                    <span class="text-slate-400">—</span>
                  {/if}
                </TableCell>
              </TableRow>
            {/each}
          </TableBody>
        </Table>
      {/if}
    </CardContent>
  </Card>
</div>

<NewProjectSheet bind:open={newSheetOpen} oncreate={refreshRecents} />

{#if staleEntry}
  <ConfirmRemoveStaleRecent
    bind:open={stalePromptOpen}
    name={staleEntry.name}
    path={staleEntry.path}
    reason={staleEntry.reason}
    onconfirm={confirmRemoveStale}
  />
{/if}
