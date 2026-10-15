<script lang="ts">
  import type { Character } from "./api";
  import { ENGINES } from "./engines";
  import { newCharacterReport } from "./wasm/palimpseste_wasm";

  let { character }: { character: Character } = $props();

  // Computed in the browser by the engine compiled to WebAssembly.
  const report = $derived(newCharacterReport(character.engine));
  const open = $derived(report.violations.filter((violation) => !violation.waived));
</script>

<li>
  <details>
    <summary>
      <strong>{character.name}</strong> · {ENGINES[character.engine]} ·
      {open.length} point{open.length > 1 ? "s" : ""} à régler
    </summary>
    <ul class="violations">
      {#each open as violation, i (i)}
        <li class={violation.category.kind}>
          <code>{violation.rule}</code>
          {violation.category.kind}
        </li>
      {/each}
    </ul>
  </details>
</li>
