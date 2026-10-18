<script lang="ts">
  // The numbered marks of the notes about one thing of the sheet. A mark
  // opens its note in a popover; the sheet also lists every note.
  import type { Note } from "./notes";

  let { notes }: { notes: Note[] } = $props();
  const uid = $props.id();
</script>

{#each notes as note (note.number)}
  <button
    type="button"
    class={["mark", note.kind]}
    popovertarget="{uid}-{note.number}"
    aria-label="Note {note.number}">{note.number}</button
  >
  <span popover id="{uid}-{note.number}" class="note">
    <strong>{note.number}. {note.kind === "waiver" ? "Dérogation" : "Ajustement"}</strong>
    {note.text}
    <span class="by">{note.author} : {note.reason}</span>
  </span>
{/each}

<style>
  .mark {
    min-width: 1.6rem;
    margin-left: 0.25rem;
    padding: 0 0.35rem;
    border: 1px solid var(--accent);
    border-radius: 1rem;
    background: none;
    color: var(--accent);
    font-size: 0.75rem;
    font-weight: bold;
    line-height: 1.4;
    vertical-align: super;
    cursor: pointer;
  }

  .mark.adjustment {
    border-style: dashed;
  }

  .note {
    max-width: min(24rem, calc(100vw - 2rem));
    padding: 1rem;
    border: 1px solid var(--muted);
    border-radius: 0.5rem;
    background: var(--page);
    color: var(--ink);
    font-size: 1rem;
    font-weight: normal;
    text-transform: none;
    letter-spacing: normal;
  }

  .note::backdrop {
    background: rgb(0 0 0 / 50%);
  }

  .by {
    display: block;
    margin-top: 0.5rem;
    color: var(--muted);
  }
</style>
