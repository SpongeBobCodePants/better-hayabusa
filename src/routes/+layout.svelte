<script lang="ts">
  import '../app.css';
  import OsGate from '$lib/components/OsGate.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Breadcrumbs from '$lib/components/Breadcrumbs.svelte';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<OsGate>
  <div class="flex h-screen">
    <Sidebar />
    <main class="flex flex-1 flex-col overflow-hidden bg-slate-50">
      <Breadcrumbs />
      <div class="flex-1 overflow-y-auto">
        {@render children()}
      </div>
    </main>
  </div>
</OsGate>
