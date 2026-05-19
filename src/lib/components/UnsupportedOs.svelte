<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  let { detectedOs }: { detectedOs: string } = $props();
</script>

<div class="flex h-screen items-center justify-center bg-slate-50 p-8">
  <div class="max-w-lg space-y-4 rounded-lg border bg-white p-8 text-center shadow-sm">
    <h1 class="text-2xl font-bold text-slate-900">Unsupported operating system</h1>
    <p class="text-slate-600">
      Better Hayabusa/ChainSaw v1 supports Windows only. You're running
      <strong>{detectedOs}</strong>.
    </p>
    <p class="text-sm text-slate-500">
      Cross-platform support is planned. Watch the GitHub project for progress.
    </p>
    <a
      href="https://github.com/MercilessSoftware/better-hayabusa-chainsaw/issues"
      class="inline-block"
      target="_blank"
      rel="noreferrer"
    >
      <Button variant="outline">Open the issue tracker</Button>
    </a>
  </div>
</div>
