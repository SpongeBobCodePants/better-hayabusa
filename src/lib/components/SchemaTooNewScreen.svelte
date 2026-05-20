<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { RELEASES_URL } from '$lib/constants';

  type Props = {
    path: string;
    name: string;
    project_version: number;
    app_version: number;
    onContinue: () => void;
  };
  let { path, name, project_version, app_version, onContinue }: Props = $props();
</script>

<div class="flex h-full items-center justify-center p-8">
  <div class="max-w-md space-y-4 rounded-lg border bg-white p-8 text-center shadow-sm">
    <h1 class="text-xl font-bold text-slate-900">
      This project was created by a newer version of Better Hayabusa
    </h1>
    <p class="text-sm text-slate-600">
      Project '<strong>{name}</strong>' was created with schema version
      <strong>{project_version}</strong>; this app supports up to version
      <strong>{app_version}</strong>. Upgrade the app to open this project.
    </p>
    <p class="text-xs text-slate-500">
      Path: <code class="rounded bg-slate-100 px-1">{path}</code>
    </p>
    <div class="flex justify-center gap-2">
      <Button href={RELEASES_URL} target="_blank" rel="noopener noreferrer" variant="outline">
        Open releases
      </Button>
      <Button onclick={onContinue}>Take me Home</Button>
    </div>
  </div>
</div>
