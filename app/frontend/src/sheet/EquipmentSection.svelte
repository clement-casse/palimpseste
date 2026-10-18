<script lang="ts">
  // The equipment in play (ADR FE-SHEET): what is worn, given by each engine
  // sheet as `worn`, and the items carried, counted up and down, or edited as
  // in the editor. It replaces the creation equipment on this device only; the
  // engine computes the values from it and reports the rules that hold in
  // play.
  import type { Snippet } from "svelte";

  import Items from "../character/Items.svelte";
  import { message } from "../messages";
  import type { ItemEntry, ReportedViolation } from "../wasm/palimpseste_wasm";
  import Inventory from "./Inventory.svelte";
  import Marks from "./Marks.svelte";
  import type { Note } from "./notes";

  let {
    notes,
    items,
    weapons,
    findings,
    kept,
    onitems,
    onforget,
    worn,
  }: {
    /** The notes of the equipment step. */
    notes: Note[];
    items: ItemEntry[];
    /** The weapons that can be added. */
    weapons: { id: string; name: string }[];
    /** What the engine reports about the equipment in play. */
    findings: ReportedViolation[];
    /** An equipment in play is kept, in place of the creation one. */
    kept: boolean;
    onitems: (items: ItemEntry[]) => void;
    onforget: () => void;
    /** What is worn: list items when read, fields when `editing`. */
    worn: Snippet<[editing: boolean]>;
  } = $props();

  let editing = $state(false);
  const names = $derived(new Map(weapons.map((weapon) => [weapon.id, weapon.name])));
  const warnings = $derived(findings.filter((finding) => !finding.waived));
</script>

<section class="block">
  <h3>Équipement<Marks {notes} /></h3>
  <button type="button" class="edit" aria-pressed={editing} onclick={() => (editing = !editing)}>
    Modifier l’équipement
  </button>
  {#if editing}
    <div class="fields">{@render worn(true)}</div>
    <Items {items} {weapons} onchange={onitems} />
  {:else}
    <ul class="listing">
      {@render worn(false)}
      <Inventory {items} {names} onchange={onitems} />
    </ul>
  {/if}

  {#if warnings.length > 0}
    <h4>À savoir</h4>
    <ul class="warnings">
      {#each warnings as warning, i (i)}
        <li>{message(warning)}</li>
      {/each}
    </ul>
  {/if}

  {#if kept}
    <p class="hint">
      Équipement de la partie, gardé sur cet appareil.
      <button
        type="button"
        onclick={() => {
          if (confirm("Reprendre l’équipement choisi à la création ?")) onforget();
        }}
      >
        Reprendre l’équipement de création
      </button>
    </p>
  {/if}
</section>

<style>
  .edit {
    min-height: 2.75rem;
    margin-bottom: 0.5rem;
  }

  .edit[aria-pressed="true"] {
    border-color: var(--accent);
    color: var(--accent);
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .warnings li {
    padding: 0.15rem 0;
    color: var(--error);
  }

  .hint button {
    margin-top: 0.25rem;
  }
</style>
