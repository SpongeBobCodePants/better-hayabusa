<script lang="ts">
  import { onMount } from 'svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
  } from '$lib/components/ui/select';
  import {
    getLaunchBehavior, setLaunchBehavior, type LaunchBehavior,
    getDefaultTimezone, setDefaultTimezone, type TimezoneMode,
    getRecentProjectsCount, setRecentProjectsCount,
  } from '$lib/ipc/app';

  let launchBehavior = $state<LaunchBehavior>('last_project');
  let tzMode = $state<TimezoneMode>('UTC');
  let count = $state<number>(5);
  let loaded = $state(false);

  const launchLabels: Record<LaunchBehavior, string> = {
    last_project: 'Open last project',
    home_page: 'Open Home page',
  };

  const tzLabels: Record<TimezoneMode, string> = {
    UTC: 'UTC',
    Local: 'Local',
  };

  onMount(async () => {
    [launchBehavior, tzMode, count] = await Promise.all([
      getLaunchBehavior(),
      getDefaultTimezone(),
      getRecentProjectsCount(),
    ]);
    loaded = true;
  });

  async function onLaunchBehaviorChange(v: string) {
    launchBehavior = v as LaunchBehavior;
    await setLaunchBehavior(launchBehavior);
  }

  async function onTzChange(v: string) {
    tzMode = v as TimezoneMode;
    await setDefaultTimezone(tzMode);
  }

  async function onCountChange() {
    const n = Math.max(1, Math.min(50, count | 0));
    count = n;
    await setRecentProjectsCount(n);
  }
</script>

<div class="mx-auto max-w-2xl space-y-6 p-8">
  <h1 class="text-2xl font-bold">Settings</h1>

  {#if loaded}
    <Card>
      <CardHeader>
        <CardTitle>General</CardTitle>
      </CardHeader>
      <CardContent class="space-y-6">
        <div class="space-y-2">
          <Label for="launch">On launch</Label>
          <Select type="single" value={launchBehavior} onValueChange={onLaunchBehaviorChange}>
            <SelectTrigger id="launch" class="w-64">
              {launchLabels[launchBehavior]}
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="last_project">Open last project</SelectItem>
              <SelectItem value="home_page">Open Home page</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="space-y-2">
          <Label for="tz">Default timezone</Label>
          <Select type="single" value={tzMode} onValueChange={onTzChange}>
            <SelectTrigger id="tz" class="w-64">
              {tzLabels[tzMode]}
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="UTC">UTC</SelectItem>
              <SelectItem value="Local">Local</SelectItem>
            </SelectContent>
          </Select>
          <p class="text-xs text-slate-500">
            All displayed timestamps use this. The DB always stores UTC.
          </p>
        </div>

        <div class="space-y-2">
          <Label for="count">Recent projects to show on Home</Label>
          <Input
            id="count"
            type="number"
            min="1"
            max="50"
            bind:value={count}
            onblur={onCountChange}
            class="w-32"
          />
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>Theme, log retention, tools</CardTitle>
      </CardHeader>
      <CardContent>
        <p class="text-sm text-slate-500">Coming in M5.</p>
      </CardContent>
    </Card>
  {/if}
</div>
