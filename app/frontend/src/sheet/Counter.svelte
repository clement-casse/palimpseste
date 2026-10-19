<script lang="ts">
  // A resource counted during a session: what is left of its maximum, spent
  // and recovered one at a time.
  import { Button, Progressbar } from "flowbite-svelte";
  import { MinusOutline, PlusOutline } from "flowbite-svelte-icons";

  import Marks from "./Marks.svelte";
  import type { Note } from "./notes";

  let {
    label,
    title,
    max,
    used,
    die,
    notes,
    onchange,
  }: {
    /** The short name, such as PV. */
    label: string;
    /** The full name. */
    title: string;
    max: number;
    used: number;
    /** The die rolled with it, such as d10. */
    die?: string;
    notes: Note[];
    onchange: (used: number) => void;
  } = $props();

  const left = $derived(max - used);
  const share = $derived(max > 0 ? (100 * left) / max : 0);
</script>

<div class="flex items-center gap-1.5" role="group" aria-label={title}>
  <span class="font-bold" {title}>{label}<Marks {notes} /></span>
  <span class="flex min-w-18 flex-col leading-tight">
    <output class="text-xl font-bold tabular-nums">{left} / {max}</output>
    {#if die}<span class="text-xs text-gray-500 dark:text-gray-400">{die}</span>{/if}
    <Progressbar progress={share} size="h-1" class="mt-1" />
  </span>
  <Button
    color="alternative"
    class="h-11 w-11 p-0 print:hidden"
    aria-label="Retirer 1 {label}"
    disabled={left <= 0}
    onclick={() => onchange(used + 1)}
  >
    <MinusOutline />
  </Button>
  <Button
    color="alternative"
    class="h-11 w-11 p-0 print:hidden"
    aria-label="Ajouter 1 {label}"
    disabled={used <= 0}
    onclick={() => onchange(used - 1)}
  >
    <PlusOutline />
  </Button>
</div>
