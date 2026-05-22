<script lang="ts">
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from '$lib/components/ui/alert-dialog';

  type Props = {
    open: boolean;
    name: string;
    path: string;
    onconfirm: () => void;
  };
  let { open = $bindable(false), name, path, onconfirm }: Props = $props();
</script>

<AlertDialog bind:open>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>Delete project '{name}'?</AlertDialogTitle>
      <AlertDialogDescription>
        This will permanently delete the entire project folder at
        <code class="rounded bg-slate-100 px-1">{path}</code>, including any
        evidence files stored inside. Files outside this folder are not
        affected. This cannot be undone.
      </AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel>Cancel</AlertDialogCancel>
      <AlertDialogAction onclick={onconfirm} class="bg-red-600 hover:bg-red-700">
        Delete
      </AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
