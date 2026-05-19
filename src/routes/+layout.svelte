<script lang="ts">
  import '../app.css';
  import OsGate from '$lib/components/OsGate.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<OsGate>
  <div class="flex h-screen">
    <Sidebar />
    <main class="flex-1 overflow-y-auto bg-slate-50">
      {@render children()}
    </main>
  </div>
</OsGate>
