<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
</script>

<div class="mx-auto max-w-3xl space-y-6 p-8">
  <header>
    <h1 class="text-3xl font-bold">Better Hayabusa</h1>
    <p class="mt-2 text-slate-600">
      A graphical UI for the Hayabusa and Chainsaw command-line forensics tools.
      Organize your investigations as projects, configure tool runs as named jobs,
      and review run history in one place.
    </p>
  </header>

  <Card>
    <CardHeader>
      <CardTitle>Get started</CardTitle>
    </CardHeader>
    <CardContent class="flex gap-3">
      <Button disabled title="Coming in M2">New project</Button>
      <Button variant="outline" disabled title="Coming in M2">Open project</Button>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Recent projects</CardTitle>
    </CardHeader>
    <CardContent>
      <p class="text-sm text-slate-500">No recent projects yet.</p>
    </CardContent>
  </Card>
</div>
