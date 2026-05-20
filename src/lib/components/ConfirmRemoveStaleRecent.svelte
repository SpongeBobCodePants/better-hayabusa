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
    reason: string;
    onconfirm: () => void;
  };
  let { open = $bindable(false), name, path, reason, onconfirm }: Props = $props();
</script>

<AlertDialog bind:open>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>Remove '{name}' from the list?</AlertDialogTitle>
      <AlertDialogDescription>
        {reason}
        Path: <code class="rounded bg-slate-100 px-1">{path}</code>
        <br /><br />
        This only removes the project from the Better Hayabusa recents
        list. Nothing on disk is changed.
      </AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel>Keep in list</AlertDialogCancel>
      <AlertDialogAction onclick={onconfirm}>Remove</AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
