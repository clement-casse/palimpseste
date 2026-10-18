<script lang="ts">
  // A value read at a glance: its short name above, the value large.
  import Marks from "./Marks.svelte";
  import type { Note } from "./notes";

  let {
    label,
    value,
    title,
    notes = [],
  }: { label: string; value: string; title?: string; notes?: Note[] } = $props();
</script>

<li class="tile" {title}>
  <span class="label">{label}<Marks {notes} /></span>
  <strong>{value}</strong>
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

  strong {
    font-size: 1.5rem;
    line-height: 1.3;
  }
</style>
