<script lang="ts">
  // The capacité whose test the dé bonus of a COC2 rank 5 goes to (ADR
  // ENG-COC-BONUS-DIE): those it offers first, the voie's others under
  // "Hors règles", which the engine then reports.
  import { Label, Select } from "flowbite-svelte";

  import { valueOf } from "../character/edits";
  import type {
    ContemporainCatalog,
    ContemporainDocument,
    RankFiveLine,
  } from "../wasm/palimpseste_wasm";
  import Ability from "./Ability.svelte";
  import { abilityLabel } from "./progression";

  let {
    choices = $bindable(),
    catalog,
    line,
  }: { choices: ContemporainDocument; catalog: ContemporainCatalog; line: RankFiveLine } = $props();

  const chosen = $derived(
    choices.bonusDieTargets?.filter((entry) => entry.voie === line.voie).at(-1)?.target ?? "",
  );
  const others = $derived(
    catalog.abilities
      .filter((a) => a.id.startsWith(`${line.voie}.`) && !line.bonusDieChoice.includes(a.id))
      .map((a) => a.id),
  );

  function choose(value: string) {
    const kept = (choices.bonusDieTargets ?? []).filter((entry) => entry.voie !== line.voie);
    choices.bonusDieTargets = value ? [...kept, { voie: line.voie, target: value }] : kept;
  }
</script>

<Label class="flex flex-col gap-2">
  <span>Capacité dont le test reçoit le dé bonus de <Ability {catalog} id={`${line.voie}.5`} /></span>
  <Select placeholder="" value={chosen} onchange={(e) => choose(valueOf(e))}>
    <option value="">— Choisir —</option>
    {#each line.bonusDieChoice as id (id)}
      <option value={id}>{abilityLabel(catalog, id)}</option>
    {/each}
    <optgroup label="Hors règles">
      {#each others as id (id)}
        <option value={id}>{abilityLabel(catalog, id)}</option>
      {/each}
    </optgroup>
  </Select>
</Label>
