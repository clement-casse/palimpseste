<script lang="ts">
  import type { Character } from "./api";
  import { ENGINES } from "./engines";
  import { characterPath, follow } from "./router.svelte";
  import { evaluate } from "./wasm/palimpseste_wasm";

  let { character }: { character: Character } = $props();

  // Computed in the browser by the engine compiled to WebAssembly.
  const open = $derived.by(() => {
    try {
      return evaluate(character.document).steps.reduce(
        (sum, step) => sum + step.incomplete + step.deviations,
        0,
      );
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
</li>

<style>
  li {
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--muted);
  }
</style>
