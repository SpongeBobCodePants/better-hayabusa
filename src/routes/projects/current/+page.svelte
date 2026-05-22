<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { currentProject } from '$lib/stores/currentProject';
  import { formatDateSync, type TimezoneMode } from '$lib/helpers/formatDate';
  import { getDefaultTimezone } from '$lib/ipc/app';

  let tzMode = $state<TimezoneMode>('UTC');

  onMount(async () => {
    tzMode = await getDefaultTimezone();
    if (!$currentProject) {
      goto('/');
    }
  });
</script>

{#if $currentProject}
  <div class="mx-auto max-w-4xl space-y-6 p-8">
    <header>
      <h1 class="text-3xl font-bold">{$currentProject.project.name}</h1>
      {#if $currentProject.project.description}
        <p class="mt-1 text-sm text-slate-600">{$currentProject.project.description}</p>
      {/if}
    </header>

    <Card>
      <CardHeader>
        <CardTitle>Project info</CardTitle>
      </CardHeader>
      <CardContent class="space-y-2 text-sm">
        <div>
          <span class="font-medium">Folder:</span>
          <code class="ml-2 rounded bg-slate-100 px-1">{$currentProject.folder_path}</code>
        </div>
        <div>
          <span class="font-medium">Created:</span>
          {formatDateSync($currentProject.project.created_at, tzMode)}
        </div>
        <div>
          <span class="font-medium">Schema version:</span>
          {$currentProject.project.app_schema_version}
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>Jobs</CardTitle>
      </CardHeader>
      <CardContent>
        <p class="mb-3 text-sm text-slate-500">No jobs yet — coming in M3.</p>
        <Button disabled>New job</Button>
      </CardContent>
    </Card>
  </div>
{/if}
