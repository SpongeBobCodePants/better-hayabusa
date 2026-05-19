<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import { detectPlatform, type SupportedPlatform } from '$lib/stores/platform';
  import UnsupportedOs from './UnsupportedOs.svelte';

  let { children }: { children: Snippet } = $props();
  let platform = $state<SupportedPlatform | null>(null);

  onMount(async () => {
    platform = await detectPlatform();
  });
</script>

{#if platform === null}
  <!-- Still detecting; render nothing to avoid flash of supported UI. -->
{:else if platform !== 'windows'}
  <UnsupportedOs detectedOs={platform} />
{:else}
  {@render children()}
{/if}
