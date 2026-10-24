<script lang="ts">
  // The characteristic a COC2 rank 5 raises: those it offers first, any
  // other under "Hors règles", which the engine then reports.
  import { Label, Select } from "flowbite-svelte";

  import { CHARACTERISTICS, valueOf } from "../character/edits";
  import type {
    Characteristic,
    ContemporainCatalog,
    ContemporainDocument,
    RankFiveLine,
  } from "../wasm/palimpseste_wasm";
  import Ability from "./Ability.svelte";

  let {
    choices = $bindable(),
    catalog,
    line,
  }: { choices: ContemporainDocument; catalog: ContemporainCatalog; line: RankFiveLine } = $props();

  const chosen = $derived(
    choices.raisedCharacteristics?.find((entry) => entry.voie === line.voie)?.characteristic ?? "",
  );
  const others = $derived(CHARACTERISTICS.filter((c) => !line.choice.includes(c)));

  function choose(value: string) {
    const kept = (choices.raisedCharacteristics ?? []).filter((entry) => entry.voie !== line.voie);
    choices.raisedCharacteristics = value
      ? [...kept, { voie: line.voie, characteristic: value as Characteristic }]
      : kept;
  }
</script>

<Label class="flex flex-col gap-2">
  <span>Caractéristique augmentée par <Ability {catalog} id={`${line.voie}.5`} /></span>
  <Select placeholder="" value={chosen} onchange={(e) => choose(valueOf(e))}>
    <option value="">— Choisir —</option>
    {#each line.choice as c (c)}
      <option value={c}>{c}</option>
    {/each}
    <optgroup label="Hors règles">
      {#each others as c (c)}
        <option value={c}>{c}</option>
      {/each}
    </optgroup>
  </Select>
</Label>
