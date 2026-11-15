<script lang="ts">
  import { A, Badge, Button, ListgroupItem } from "flowbite-svelte";

  import type { Character } from "$lib/api";
  import { ENGINES, unsettledAll } from "$lib/engines";
  import { characterPath, characterSheetPath, follow } from "$lib/router.svelte";
  import { attempt, evaluate } from "$lib/engine";
  import { COC } from "$lib/engine/ids";

  let { character }: { character: Character } = $props();

  // Computed in the browser by the engine compiled to WebAssembly.
  const evaluation = $derived.by(() => {
    const result = attempt(() => evaluate(character.document));
    return result.ok ? result.value : null;
  });
  const open = $derived(evaluation ? unsettledAll(evaluation) : null);
  /** Where the character stands after creation: its level, or the XP left. */
  const standing = $derived.by(() => {
    const details = evaluation?.details;
    if (details?.engine === "fantasy") return `Niv. ${details.level}`;
    const xp = evaluation?.values.find((line) => line.id === COC.experience)?.total;
    return xp === undefined ? null : `XP ${xp}`;
  });
</script>

<ListgroupItem class="flex-wrap gap-x-3 gap-y-2 py-3">
  <A href={characterPath(character.id)} onclick={follow} class="font-semibold">{character.name}</A>
  <Badge color="gray">{ENGINES[character.engine]}</Badge>
  {#if standing}<Badge color="gray">{standing}</Badge>{/if}
  {#if open === null}
    <Badge color="red">document illisible</Badge>
  {:else if open === 0}
    <Badge color="green">terminé</Badge>
  {:else}
    <Badge color="yellow">{open} point{open > 1 ? "s" : ""} à régler</Badge>
  {/if}
  <Button
    size="xs"
    color="alternative"
    class="ms-auto"
    href={characterSheetPath(character.id)}
    onclick={follow}
  >
    Fiche
  </Button>
</ListgroupItem>
