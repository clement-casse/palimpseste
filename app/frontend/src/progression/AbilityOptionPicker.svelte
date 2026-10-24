<script lang="ts">
  // The option of a COF2 capacité held: those it offers first, any other
  // under "Hors règles", which the engine then reports (ADR FE-OVERRIDES).
  import { Label, Select } from "flowbite-svelte";

  import { valueOf } from "../character/edits";
  import type { AbilityOptionLine, FantasyCatalog, FantasyDocument } from "../wasm/palimpseste_wasm";
  import Ability from "./Ability.svelte";

  let {
    choices = $bindable(),
    catalog,
    line,
  }: { choices: FantasyDocument; catalog: FantasyCatalog; line: AbilityOptionLine } = $props();

  const chosen = $derived(
    choices.abilityOptions?.find((entry) => entry.ability === line.ability)?.option ?? "",
  );
  const others = $derived(catalog.abilityOptions.filter((o) => !line.offered.includes(o.id)));
  const names = $derived(new Map(catalog.abilityOptions.map((o) => [o.id, o.name])));

  function choose(option: string) {
    const kept = (choices.abilityOptions ?? []).filter((entry) => entry.ability !== line.ability);
    choices.abilityOptions = option ? [...kept, { ability: line.ability, option }] : kept;
  }
</script>

<Label class="flex flex-col gap-2">
  <span>Option de <Ability {catalog} id={line.ability} /></span>
  <Select placeholder="" value={chosen} onchange={(e) => choose(valueOf(e))}>
    <option value="">{line.required ? "— Choisir —" : "— Sans option —"}</option>
    {#each line.offered as id (id)}
      <option value={id}>{names.get(id) ?? id}</option>
    {/each}
    <optgroup label="Hors règles">
      {#each others as option (option.id)}
        <option value={option.id}>{option.name}</option>
      {/each}
    </optgroup>
  </Select>
</Label>
