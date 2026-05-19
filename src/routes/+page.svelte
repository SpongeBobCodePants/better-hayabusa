<script lang="ts">
  import { onMount } from 'svelte';
  import { getAppVersion } from '$lib/ipc/app';

  let version = $state<string | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const v = await getAppVersion();
      version = v.version;
    } catch (e) {
      error = String(e);
    }
  });
</script>

<main class="p-8">
  <h1 class="text-2xl font-bold">Better Hayabusa/ChainSaw</h1>
  <p class="text-sm text-slate-500">Smoke test</p>
  {#if version}
    <p class="mt-4">App version: <code>{version}</code></p>
  {:else if error}
    <p class="mt-4 text-red-500">Error: {error}</p>
  {:else}
    <p class="mt-4 text-slate-400">Loading…</p>
  {/if}
</main>
