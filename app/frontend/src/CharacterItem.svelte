<script lang="ts">
  import type { Character } from "./api";
  import { ENGINES, unsettled } from "./engines";
  import { characterPath, characterSheetPath, follow } from "./router.svelte";
  import { evaluate } from "./wasm/palimpseste_wasm";

  let { character }: { character: Character } = $props();

  // Computed in the browser by the engine compiled to WebAssembly.
  const open = $derived.by(() => {
    try {
      return unsettled(evaluate(character.document).steps);
    } catch {
      return null;
    }
  });
</script>

<li>
  <a href={characterPath(character.id)} onclick={follow}>
    <strong>{character.name}</strong>
  </a>
  · {ENGINES[character.engine]} ·
  {#if open === null}
    document illisible
  {:else if open === 0}
    terminé
  {:else}
    {open} point{open > 1 ? "s" : ""} à régler
  {/if}
  · <a href={characterSheetPath(character.id)} onclick={follow}>Fiche</a>
</li>

<style>
  li {
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--muted);
  }
</style>
