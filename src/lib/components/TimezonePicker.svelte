<script lang="ts">
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import { Popover, PopoverContent, PopoverTrigger } from '$lib/components/ui/popover';
  import {
    Command,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
  } from '$lib/components/ui/command';
  import { cn } from '$lib/utils';
  import { getAllTimezones, formatTimezoneLabel } from '$lib/helpers/timezones';

  type Props = {
    value: string;
    onValueChange: (v: string) => void;
  };

  let { value, onValueChange }: Props = $props();

  let open = $state(false);
  const options = getAllTimezones();

  function pick(v: string) {
    onValueChange(v);
    open = false;
  }
</script>

<Popover bind:open>
  <PopoverTrigger
    class={cn(
      'border-input dark:bg-input/30 dark:hover:bg-input/50 focus-visible:border-ring focus-visible:ring-ring/50',
      'flex h-8 w-96 items-center justify-between gap-1.5 rounded-lg border bg-transparent py-2 pr-2 pl-2.5',
      'text-sm whitespace-nowrap transition-colors select-none outline-none focus-visible:ring-3',
      'disabled:cursor-not-allowed disabled:opacity-50',
    )}
    aria-label="Default timezone"
  >
    <span class="line-clamp-1 text-left">{formatTimezoneLabel(value)}</span>
    <ChevronDownIcon class="text-muted-foreground pointer-events-none size-4 shrink-0" />
  </PopoverTrigger>
  <PopoverContent class="w-96 p-0" align="start">
    <Command>
      <CommandInput placeholder="Search timezones..." />
      <CommandList class="max-h-72 overflow-y-auto">
        <CommandEmpty>No timezone found.</CommandEmpty>
        <CommandGroup>
          {#each options as opt (opt.value)}
            <CommandItem value={opt.label} onSelect={() => pick(opt.value)}>
              {opt.label}
            </CommandItem>
          {/each}
        </CommandGroup>
      </CommandList>
    </Command>
  </PopoverContent>
</Popover>
