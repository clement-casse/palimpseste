<script lang="ts">
  // Every waiver and adjustment of the character, numbered as their marks.
  import type { Note } from "./notes";

  let { notes }: { notes: Note[] } = $props();
</script>

{#if notes.length > 0}
  <section class="block">
    <h3>Dérogations et ajustements</h3>
    <ol>
      {#each notes as note (note.number)}
        <li class={{ unplaced: note.place === null }}>
          <span class="number">{note.number}</span>
          <span>
            <strong>{note.kind === "waiver" ? "Dérogation" : "Ajustement"}</strong>
            {note.text}
            <span class="by">{note.author} : {note.reason}</span>
          </span>
        </li>
      {/each}
    </ol>
  </section>
{/if}

<style>
  ol {
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    gap: 0.5rem;
    padding: 0.3rem 0;
  }

  .number {
    flex: none;
    min-width: 1.6rem;
    height: fit-content;
    border: 1px solid var(--accent);
    border-radius: 1rem;
    color: var(--accent);
    font-size: 0.75rem;
    font-weight: bold;
    text-align: center;
  }

  .unplaced {
    color: var(--muted);
  }

  .by {
    display: block;
    color: var(--muted);
    font-size: 0.9em;
  }
</style>
