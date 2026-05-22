<script lang="ts">
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { listAllProjects } from '$lib/ipc/projects';
  import { getDefaultTimezone, type TimezoneMode } from '$lib/ipc/app';
  import ProjectsTable from '$lib/components/ProjectsTable.svelte';
  import NewProjectSheet from '$lib/components/NewProjectSheet.svelte';
  import type { RecentProjectListEntry } from '$lib/generated/RecentProjectListEntry';

  let projects = $state<RecentProjectListEntry[]>([]);
  let tzMode = $state<TimezoneMode>('UTC');
  let loaded = $state(false);
  let newSheetOpen = $state(false);

  async function refresh() {
    [projects, tzMode] = await Promise.all([listAllProjects(), getDefaultTimezone()]);
    loaded = true;
  }

  onMount(refresh);
</script>

<div class="mx-auto max-w-5xl space-y-6 p-8">
  <header class="flex items-center justify-between">
    <h1 class="text-2xl font-bold">Projects</h1>
    <Button onclick={() => (newSheetOpen = true)}>New project</Button>
  </header>

  {#if !loaded}
    <p class="text-sm text-slate-500">Loading...</p>
  {:else if projects.length === 0}
    <div class="flex flex-col items-center gap-3 py-12 text-center">
      <img src="/img/no-projects-placeholder.svg" alt="No projects" class="h-40 w-40" />
      <p class="text-base font-medium text-slate-700">
        You have no projects. Looks like somebody better get off their ass and
        GET TO WORK!
      </p>
    </div>
  {:else}
    <ProjectsTable {projects} {tzMode} onchange={refresh} />
  {/if}
</div>

<NewProjectSheet bind:open={newSheetOpen} oncreate={refresh} />
