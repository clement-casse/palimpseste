<script lang="ts">
  import type { Character } from "./api";
  import { ENGINES } from "./engines";
  import { follow } from "./router.svelte";
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
  <a href="/characters/{character.id}" onclick={follow}>
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
