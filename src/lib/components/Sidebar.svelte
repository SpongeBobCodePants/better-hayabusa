<script lang="ts">
  import { page } from '$app/state';
  import { Separator } from '$lib/components/ui/separator';
  import SidebarFooter from './SidebarFooter.svelte';
  import { currentProject } from '$lib/stores/currentProject';

  type NavItem = { href: string; label: string };
  const topItems: NavItem[] = [
    { href: '/', label: 'Home' },
    { href: '/projects', label: 'Projects' }
  ];
  const toolsItems: NavItem[] = [
    { href: '/tools/hayabusa', label: 'Hayabusa' },
    { href: '/tools/chainsaw', label: 'Chainsaw' }
  ];
  const bottomItems: NavItem[] = [{ href: '/settings', label: 'Settings' }];

  // Bug fix: previously startsWith match caused /settings/about to highlight
  // Settings. Now we require an exact match OR a strict subpath match for
  // items that legitimately have children (currently only /tools/*).
  function isActive(href: string) {
    const path = page.url.pathname;
    if (href === '/') return path === '/';
    if (href === '/settings') return path === '/settings'; // exact only
    if (href === '/projects') return path === '/projects' || path.startsWith('/projects/');
    if (href.startsWith('/tools/')) return path === href;
    return path === href;
  }
</script>

<aside class="flex h-screen w-56 flex-col border-r bg-white">
  <nav class="flex-1 overflow-y-auto py-4">
    {#each topItems as item}
      <a
        href={item.href}
        class="block px-4 py-2 text-sm hover:bg-slate-100"
        class:bg-slate-100={isActive(item.href)}
        class:font-medium={isActive(item.href)}
      >
        {item.label}
      </a>
    {/each}

    {#if $currentProject}
      <Separator class="my-2" />
      <div class="px-4 py-1 text-xs font-semibold uppercase text-slate-500">
        Project
      </div>
      <a
        href="/projects/current"
        class="block px-4 py-2 text-sm hover:bg-slate-100"
        class:bg-slate-100={page.url.pathname === '/projects/current'}
        class:font-medium={page.url.pathname === '/projects/current'}
      >
        ▸ {$currentProject.project.name}
      </a>
    {/if}

    <Separator class="my-2" />

    <div class="px-4 py-1 text-xs font-semibold uppercase text-slate-500">Tools</div>
    {#each toolsItems as item}
      <a
        href={item.href}
        class="block px-4 py-2 text-sm hover:bg-slate-100"
        class:bg-slate-100={isActive(item.href)}
        class:font-medium={isActive(item.href)}
      >
        {item.label}
      </a>
    {/each}

    <Separator class="my-2" />

    {#each bottomItems as item}
      <a
        href={item.href}
        class="block px-4 py-2 text-sm hover:bg-slate-100"
        class:bg-slate-100={isActive(item.href)}
        class:font-medium={isActive(item.href)}
      >
        {item.label}
      </a>
    {/each}
  </nav>

  <SidebarFooter />
</aside>
