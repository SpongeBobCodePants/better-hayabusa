<script lang="ts">
  import { onMount } from 'svelte';
  import { getAppVersion } from '$lib/ipc/app';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';

  let version = $state<string>('…');

  onMount(async () => {
    try {
      const v = await getAppVersion();
      version = v.version;
    } catch {
      version = '?';
    }
  });

  const acknowledgements = [
    { name: 'Hayabusa', url: 'https://github.com/Yamato-Security/hayabusa' },
    { name: 'Chainsaw', url: 'https://github.com/WithSecureLabs/chainsaw' },
    { name: 'SigmaHQ Sigma rules', url: 'https://github.com/SigmaHQ/sigma' },
    { name: 'Tauri', url: 'https://tauri.app' },
    { name: 'Svelte', url: 'https://svelte.dev' },
    { name: 'shadcn-svelte', url: 'https://www.shadcn-svelte.com' }
  ];
</script>

<div class="mx-auto max-w-2xl space-y-6 p-8">
  <header>
    <h1 class="text-3xl font-bold">Better Hayabusa</h1>
    <p class="mt-1 text-lg italic text-slate-600">Making your life suck less…</p>
  </header>

  <Card>
    <CardHeader>
      <CardTitle>About</CardTitle>
    </CardHeader>
    <CardContent class="space-y-3 text-sm">
      <div>
        <span class="font-medium">Version:</span> <code>{version}</code>
      </div>
      <div>
        <span class="font-medium">License:</span> MIT
        <a href="https://opensource.org/licenses/MIT" target="_blank" rel="noopener noreferrer" class="ml-2 text-blue-600 underline">
          read
        </a>
      </div>
      <div>
        <span class="font-medium">Source:</span>
        <a
          href="https://github.com/MercilessSoftware/better-hayabusa-chainsaw"
          target="_blank"
          rel="noopener noreferrer"
          class="text-blue-600 underline"
        >
          GitHub
        </a>
      </div>
      <div class="pt-2 text-xs text-slate-500">© 2026 Merciless Software</div>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Acknowledgements</CardTitle>
    </CardHeader>
    <CardContent>
      <ul class="space-y-1 text-sm">
        {#each acknowledgements as ack}
          <li>
            <a href={ack.url} target="_blank" rel="noopener noreferrer" class="text-blue-600 underline">
              {ack.name}
            </a>
          </li>
        {/each}
      </ul>
    </CardContent>
  </Card>
</div>
