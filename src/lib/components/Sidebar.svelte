<script lang="ts">
  import { page } from '$app/state';
  import { Separator } from '$lib/components/ui/separator';
  import SidebarFooter from './SidebarFooter.svelte';

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

  function isActive(href: string) {
    if (href === '/') return page.url.pathname === '/';
    return page.url.pathname.startsWith(href);
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
