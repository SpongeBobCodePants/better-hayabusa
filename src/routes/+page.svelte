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
  import { listRecentProjects, removeRecentProject } from '$lib/ipc/projects';
  import { openAndInstall } from '$lib/stores/currentProject';
  import { formatDateSync } from '$lib/helpers/formatDate';
  import { getDefaultTimezone, getRecentProjectsCount, type TimezoneMode } from '$lib/ipc/app';
  import NewProjectSheet from '$lib/components/NewProjectSheet.svelte';
  import ConfirmRemoveStaleRecent from '$lib/components/ConfirmRemoveStaleRecent.svelte';
  import type { RecentProject } from '$lib/generated/RecentProject';

  let recents = $state<RecentProject[]>([]);
  let tzMode = $state<TimezoneMode>('UTC');
  let count = $state<number>(5);
  let loaded = $state(false);
  let newSheetOpen = $state(false);
  let stalePromptOpen = $state(false);
  let staleEntry = $state<{ path: string; name: string; reason: string } | null>(null);

  onMount(async () => {
    [recents, tzMode, count] = await Promise.all([
      listRecentProjects(),
      getDefaultTimezone(),
      getRecentProjectsCount(),
    ]);
    loaded = true;
  });

  async function handleOpenRecent(r: RecentProject) {
    const result = await openAndInstall(r.path);
    if (result.kind === 'Loaded') {
      goto('/projects/current');
    } else if (result.kind === 'SchemaTooNew') {
      // SchemaTooNew handled by the sticky-fail/upgrade screen elsewhere.
      // For now, surface an alert. Improve in Task 25 once screen exists.
      alert(`Project requires app v${result.app_version}+; you have v${result.project_version}.`);
    } else if (result.kind === 'Missing') {
      // Folder is gone — prompt the user before removing the recents row.
      staleEntry = { path: result.path, name: result.name, reason: result.reason };
      stalePromptOpen = true;
    } else if (result.kind === 'Failed') {
      // Sticky-restore branch — not expected here, but be defensive.
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

  function refreshRecents() {
    listRecentProjects().then((r) => (recents = r));
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
        disabled={loaded && recents.length === 0}
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
      {:else if recents.length === 0}
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
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Last opened</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {#each recents.slice(0, count) as r}
              <TableRow
                class="cursor-pointer hover:bg-slate-100"
                onclick={() => handleOpenRecent(r)}
              >
                <TableCell class="font-medium">{r.name}</TableCell>
                <TableCell class="text-slate-500">
                  {formatDateSync(r.last_opened_at, tzMode)}
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
