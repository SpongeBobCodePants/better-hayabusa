<script lang="ts">
  import { Button } from '$lib/components/ui/button';

  type Props = {
    path: string;
    name: string;
    reason: string;
    onContinue: () => void;
  };
  let { path, name, reason, onContinue }: Props = $props();
</script>

<div class="flex h-full items-center justify-center p-8">
  <div class="max-w-md space-y-4 rounded-lg border bg-white p-8 text-center shadow-sm">
    <img
      src="/img/sticky-fail-placeholder.svg"
      alt="Project failed to load"
      class="mx-auto h-32 w-32"
    />
    <h1 class="text-xl font-bold text-slate-900">
      Something broke, and we can't find data associated with this project
    </h1>
    <ul class="space-y-1 text-left text-sm text-slate-600">
      <li><span class="font-medium">Project:</span> {name}</li>
      <li><span class="font-medium">Path:</span> <code class="rounded bg-slate-100 px-1">{path}</code></li>
      <li><span class="font-medium">Reason:</span> {reason}</li>
    </ul>
    <Button onclick={onContinue}>OK, take me Home</Button>
  </div>
</div>
