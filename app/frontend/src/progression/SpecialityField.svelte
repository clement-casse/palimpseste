<script lang="ts">
  // The speciality a capacité held asks for, written freely (ADR
  // ENG-SPECIALITIES); clearing it gives it up.
  import { Input, Label } from "flowbite-svelte";

  import { valueOf } from "../character/edits";
  import type { SpecialityEntry } from "../wasm/palimpseste_wasm";
  import Ability from "./Ability.svelte";
  import type { Abilities } from "./progression";

  let {
    choices = $bindable(),
    catalog,
    ability,
  }: { choices: { specialities?: SpecialityEntry[] }; catalog: Abilities; ability: string } =
    $props();

  const asked = $derived(catalog.abilities.find((a) => a.id === ability)?.speciality ?? "");
  const written = $derived(
    choices.specialities?.find((entry) => entry.ability === ability)?.speciality ?? "",
  );

  function write(text: string) {
    const kept = (choices.specialities ?? []).filter((entry) => entry.ability !== ability);
    const speciality = text.trim();
    choices.specialities = speciality ? [...kept, { ability, speciality }] : kept;
  }
</script>

<Label class="flex flex-col gap-2">
  <span>{asked.charAt(0).toUpperCase() + asked.slice(1)} : <Ability {catalog} id={ability} /></span>
  <Input value={written} placeholder={asked} onchange={(e) => write(valueOf(e))} />
</Label>
