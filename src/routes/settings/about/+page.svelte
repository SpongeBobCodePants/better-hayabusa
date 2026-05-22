<script lang="ts">
  import { onMount } from 'svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { getAppVersion } from '$lib/ipc/app';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { REPO_URL } from '$lib/constants';

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
    { name: 'Yamato Security (Hayabusa)', url: 'https://github.com/Yamato-Security/hayabusa' },
    { name: 'WithSecureLabs (Chainsaw)', url: 'https://github.com/WithSecureLabs/chainsaw' },
    { name: 'Yamato Security (Takajo)', url: 'https://github.com/Yamato-Security/takajo' },
    { name: 'SigmaHQ (Sigma rules)', url: 'https://github.com/SigmaHQ/sigma' },
    { name: 'Tauri', url: 'https://tauri.app' },
    { name: 'Svelte', url: 'https://svelte.dev' },
    { name: 'shadcn-svelte', url: 'https://www.shadcn-svelte.com' }
  ];

  function open(url: string) {
    void openUrl(url);
  }
</script>

<div class="mx-auto max-w-2xl space-y-6 p-8">
  <header>
    <h1 class="text-3xl font-bold">Better Hayabusa</h1>
    <p class="mt-1 text-lg italic text-slate-600">Making your life suck a little less…</p>
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
        <span class="font-medium">License:</span> AGPL-3.0-or-later
        <button
          type="button"
          onclick={() => open('https://www.gnu.org/licenses/agpl-3.0.html')}
          class="ml-2 text-blue-600 underline"
        >
          read
        </button>
      </div>
      <div>
        <span class="font-medium">Source:</span>
        <button
          type="button"
          onclick={() => open(REPO_URL)}
          class="text-blue-600 underline"
        >
          GitHub
        </button>
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
            <button
              type="button"
              onclick={() => open(ack.url)}
              class="text-blue-600 underline"
            >
              {ack.name}
            </button>
          </li>
        {/each}
      </ul>
    </CardContent>
  </Card>
</div>
