<script lang="ts">
  import '../app.css';
  import OsGate from '$lib/components/OsGate.svelte';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<OsGate>
  {@render children()}
</OsGate>
