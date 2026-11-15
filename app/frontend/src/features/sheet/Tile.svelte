<script lang="ts">
  // A value read at a glance: its short name above, the value large. With
  // its breakdown, the value shows it in a popover while hovered or focused,
  // so what makes it (an armour, a capacité, an adjustment) can be read.
  import { Button, Card } from "flowbite-svelte";

  import BreakdownPopover from "$lib/components/BreakdownPopover.svelte";
  import type { ContributionLine, SituationalLine } from "$lib/engine";
  import Marks from "$lib/components/Marks.svelte";
  import type { Note } from "$lib/character/notes";
  import RollButton from "./RollButton.svelte";

  let {
    label,
    value,
    title,
    notes = [],
    contributions,
    situational = [],
    onroll,
    rollLabel,
  }: {
    label: string;
    value: string;
    title?: string;
    notes?: Note[];
    contributions?: ContributionLine[];
    /** Its modifiers holding only in a situation, in its popover. */
    situational?: SituationalLine[];
    /** Opens the roll of the value; none for a value not rolled, or a sheet only read. */
    onroll?: () => void;
    /** What the roll is, such as `Test de FOR`; the title or the label otherwise. */
    rollLabel?: string;
  } = $props();

  const uid = $props.id();
  const id = `tile-${uid}`;
</script>

<li {title}>
  <Card class="h-full items-center px-1 py-2 text-center">
    <span class="text-xs leading-tight text-gray-500 dark:text-gray-400">{label}<Marks {notes} /></span>
    {#if contributions && contributions.length > 0}
      <Button
        {id}
        color="light"
        class="min-w-11 border-0 px-1 py-0 text-2xl font-bold underline decoration-dotted underline-offset-4"
        aria-label="{title ?? label} {value}, détail"
      >{value}</Button
      >
      <BreakdownPopover triggeredBy="#{id}" title="{title ?? label} : {value}" {contributions} {situational} />
    {:else}
      <strong class="text-2xl font-bold text-gray-900 dark:text-white">{value}</strong>
    {/if}
    {#if onroll}<RollButton label={rollLabel ?? title ?? label} onclick={onroll} />{/if}
  </Card>
</li>
