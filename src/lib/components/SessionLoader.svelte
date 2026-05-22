<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { checkLastOpenProject } from '$lib/ipc/projects';
  import { loadCurrentProject } from '$lib/stores/currentProject';
  import StickyFailScreen from './StickyFailScreen.svelte';
  import SchemaTooNewScreen from './SchemaTooNewScreen.svelte';
  import type { LaunchResult } from '$lib/generated/LaunchResult';

  type Props = { children: Snippet };
  let { children }: Props = $props();

  type State =
    | { kind: 'loading' }
    | { kind: 'ready' }
    | { kind: 'sticky_fail'; path: string; name: string; reason: string }
    | { kind: 'schema_too_new'; path: string; name: string; project_version: number; app_version: number };

  let state = $state<State>({ kind: 'loading' });

  onMount(async () => {
    let result: LaunchResult;
    try {
      result = await checkLastOpenProject();
    } catch (e) {
      // If the startup IPC errors (e.g. transient DB read failure),
      // fall through to Home instead of leaving the spinner up forever.
      console.error('checkLastOpenProject failed at startup:', e);
      state = { kind: 'ready' };
      return;
    }
    switch (result.kind) {
      case 'Loaded':
        await loadCurrentProject(); // hydrate store from backend
        state = { kind: 'ready' };
        goto('/projects/current');
        break;
      case 'Failed':
        state = {
          kind: 'sticky_fail',
          path: result.path,
          name: result.name,
          reason: result.reason,
        };
        break;
      case 'SchemaTooNew':
        state = {
          kind: 'schema_too_new',
          path: result.path,
          name: result.name,
          project_version: result.project_version,
          app_version: result.app_version,
        };
        break;
      case 'NoneSet':
      case 'Disabled':
        state = { kind: 'ready' };
        break;
    }
  });

  function dismissToHome() {
    state = { kind: 'ready' };
    goto('/');
  }
</script>

{#if state.kind === 'loading'}
  <div class="flex h-full items-center justify-center text-sm text-slate-500">
    Loading...
  </div>
{:else if state.kind === 'sticky_fail'}
  <StickyFailScreen
    path={state.path}
    name={state.name}
    reason={state.reason}
    onContinue={dismissToHome}
  />
{:else if state.kind === 'schema_too_new'}
  <SchemaTooNewScreen
    path={state.path}
    name={state.name}
    project_version={state.project_version}
    app_version={state.app_version}
    onContinue={dismissToHome}
  />
{:else}
  {@render children()}
{/if}
