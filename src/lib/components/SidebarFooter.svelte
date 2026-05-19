<script lang="ts">
  import { onMount } from 'svelte';
  import { getAppVersion } from '$lib/ipc/app';

  let version = $state<string>('…');

  onMount(async () => {
    try {
      const v = await getAppVersion();
      version = v.version;
    } catch {
      version = '?';
    }
  });
</script>

<a
  href="/settings/about"
  class="block w-full px-4 py-3 text-left text-xs text-slate-500 hover:bg-slate-100"
>
  <div class="font-medium text-slate-700">Merciless Software</div>
  <div>v{version}</div>
</a>
