<script lang="ts">
  import { page } from '$app/state';
  import { currentProject } from '$lib/stores/currentProject';
  import {
    Breadcrumb,
    BreadcrumbList,
    BreadcrumbItem,
    BreadcrumbLink,
    BreadcrumbPage,
    BreadcrumbSeparator,
  } from '$lib/components/ui/breadcrumb';

  type Crumb = { label: string; href: string | null };

  function deriveCrumbs(pathname: string, projectName: string | undefined): Crumb[] {
    const crumbs: Crumb[] = [{ label: 'Home', href: '/' }];

    if (pathname === '/') {
      crumbs[0].href = null; // current page
      return crumbs;
    }

    const parts = pathname.split('/').filter(Boolean);

    if (parts[0] === 'projects') {
      crumbs.push({ label: 'Projects', href: '/projects/' });
      if (parts[1] === 'current') {
        crumbs.push({ label: projectName ?? 'Project', href: null });
      } else if (parts.length === 1) {
        crumbs[crumbs.length - 1].href = null;
      }
    } else if (parts[0] === 'settings') {
      crumbs.push({ label: 'Settings', href: '/settings' });
      if (parts[1] === 'about') {
        crumbs.push({ label: 'About', href: null });
      } else {
        crumbs[crumbs.length - 1].href = null;
      }
    } else if (parts[0] === 'tools') {
      crumbs.push({ label: 'Tools', href: null });
      if (parts[1] === 'hayabusa') {
        crumbs.push({ label: 'Hayabusa', href: null });
      } else if (parts[1] === 'chainsaw') {
        crumbs.push({ label: 'Chainsaw', href: null });
      }
    }
    return crumbs;
  }

  let crumbs = $derived(deriveCrumbs(page.url.pathname, $currentProject?.project.name));
</script>

<Breadcrumb class="border-b bg-white px-6 py-3">
  <BreadcrumbList>
    {#each crumbs as crumb, i}
      <BreadcrumbItem>
        {#if crumb.href}
          <BreadcrumbLink href={crumb.href}>{crumb.label}</BreadcrumbLink>
        {:else}
          <BreadcrumbPage>{crumb.label}</BreadcrumbPage>
        {/if}
      </BreadcrumbItem>
      {#if i < crumbs.length - 1}
        <BreadcrumbSeparator />
      {/if}
    {/each}
  </BreadcrumbList>
</Breadcrumb>
