<script lang="ts">
  // A resource counted during a session: what is left of its maximum, spent
  // and recovered one at a time.
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

<div class="counter" role="group" aria-label={title}>
  <span class="label" {title}>{label}<Marks {notes} /></span>
  <span class="count">
    <output>{left} / {max}</output>
    {#if die}<span class="die">{die}</span>{/if}
    <span class="gauge" aria-hidden="true"><span style:width="{share}%"></span></span>
  </span>
  <button type="button" aria-label="Retirer 1 {label}" disabled={left <= 0} onclick={() => onchange(used + 1)}>
    −
  </button>
  <button type="button" aria-label="Ajouter 1 {label}" disabled={used <= 0} onclick={() => onchange(used - 1)}>
    +
  </button>
</div>

<style>
  .counter {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }

  .label {
    font-weight: bold;
  }

  .count {
    display: flex;
    flex-direction: column;
    min-width: 4.5rem;
    line-height: 1.2;
  }

  output {
    font-size: 1.25rem;
    font-weight: bold;
    font-variant-numeric: tabular-nums;
  }

  .die {
    color: var(--muted);
    font-size: 0.8rem;
  }

  .gauge {
    height: 0.25rem;
    margin-top: 0.2rem;
    border-radius: 0.25rem;
    background: color-mix(in srgb, var(--muted) 35%, transparent);
  }

  .gauge span {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--accent);
  }

  button {
    width: 2.75rem;
    height: 2.75rem;
    padding: 0;
    font-size: 1.4rem;
    line-height: 1;
  }
</style>
