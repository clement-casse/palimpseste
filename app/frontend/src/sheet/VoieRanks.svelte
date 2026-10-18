<script lang="ts">
  // The voies of a character with the ranks acquired in each. Ranks beyond
  // 1 to 5 are written out, since nothing hides what the document holds.
  import type { VoieEntry } from "../wasm/palimpseste_wasm";
  import Marks from "./Marks.svelte";
  import { type Note, marksAt } from "./notes";

  let {
    voies,
    names,
    notes,
  }: { voies: VoieEntry[]; names: Map<string, string>; notes: Note[] } = $props();

  const RANKS = [1, 2, 3, 4, 5];
</script>

{#if voies.some((voie) => voie.ranks.length > 0)}
  <ul class="voie-ranks">
    {#each voies.filter((voie) => voie.ranks.length > 0) as voie (voie.id)}
      {@const others = voie.ranks.filter((rank) => !RANKS.includes(rank))}
      <li>
        <span>
          {names.get(voie.id) ?? voie.id}<Marks notes={marksAt(notes, { kind: "voie", id: voie.id })} />
        </span>
        <span class="ranks">
          <span class="visually-hidden">Rangs {voie.ranks.join(", ")}</span>
          {#each RANKS as rank (rank)}
            <span class={["pip", voie.ranks.includes(rank) && "acquired"]} aria-hidden="true"></span>
          {/each}
          {#if others.length > 0}<span aria-hidden="true">+ rang {others.join(", ")}</span>{/if}
        </span>
      </li>
    {/each}
  </ul>
{:else}
  <p class="hint">Aucune voie.</p>
{/if}

<style>
  li {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem 0.5rem;
    align-items: center;
    justify-content: space-between;
    padding: 0.3rem 0;
    border-bottom: 1px solid color-mix(in srgb, var(--muted) 30%, transparent);
  }

  .ranks {
    display: inline-flex;
    gap: 0.3rem;
    align-items: center;
    color: var(--muted);
    font-size: 0.85rem;
  }

  .pip {
    width: 0.8rem;
    height: 0.8rem;
    border: 1.5px solid var(--muted);
    border-radius: 50%;
  }

  .pip.acquired {
    border-color: var(--accent);
    background: var(--accent);
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
</style>
