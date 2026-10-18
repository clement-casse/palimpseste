<script lang="ts">
  // A value read at a glance: its short name above, the value large. With
  // its breakdown, the value opens it, so what makes it (an armour, a
  // capacité, an adjustment) can be read.
  import Breakdown from "../character/Breakdown.svelte";
  import type { ContributionLine } from "../wasm/palimpseste_wasm";
  import Marks from "./Marks.svelte";
  import type { Note } from "./notes";

  let {
    label,
    value,
    title,
    notes = [],
    contributions,
  }: {
    label: string;
    value: string;
    title?: string;
    notes?: Note[];
    contributions?: ContributionLine[];
  } = $props();
  const uid = $props.id();
</script>

<li class="tile" {title}>
  <span class="label">{label}<Marks {notes} /></span>
  {#if contributions && contributions.length > 0}
    <button type="button" class="value" popovertarget={uid} aria-label="{title ?? label} {value}, détail"
      >{value}</button
    >
    <div popover id={uid} class="breakdown">
      <strong>{title ?? label} : {value}</strong>
      <Breakdown {contributions} />
    </div>
  {:else}
    <strong class="value">{value}</strong>
  {/if}
</li>

<style>
  .tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0.4rem 0.25rem;
    border: 1px solid color-mix(in srgb, var(--muted) 45%, transparent);
    border-radius: 0.5rem;
    text-align: center;
  }

  .label {
    color: var(--muted);
    font-size: 0.8rem;
    line-height: 1.2;
  }

  .value {
    font-size: 1.5rem;
    font-weight: bold;
    line-height: 1.3;
  }

  button.value {
    min-width: 2.75rem;
    padding: 0 0.25rem;
    border: none;
    border-bottom: 1px dotted var(--muted);
    border-radius: 0;
    background: none;
    color: inherit;
    cursor: pointer;
  }

  .breakdown {
    min-width: min(18rem, calc(100vw - 2rem));
    max-width: min(24rem, calc(100vw - 2rem));
    padding: 1rem;
    border: 1px solid var(--muted);
    border-radius: 0.5rem;
    background: var(--page);
    color: var(--ink);
    text-align: left;
  }

  .breakdown::backdrop {
    background: rgb(0 0 0 / 50%);
  }
</style>
