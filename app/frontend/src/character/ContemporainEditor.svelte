<script lang="ts">
  // The choices of a COC2 character, one section per creation step. The book
  // allows deferring and revisiting every choice; budgets and limits come
  // from the engine, which reports what goes beyond them.
  import {
    Badge,
    Checkbox,
    Heading,
    Helper,
    Input,
    Label,
    Listgroup,
    ListgroupItem,
    P,
    Select,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHeadCell,
  } from "flowbite-svelte";

  import type {
    Characteristic,
    ContemporainCatalog,
    ContemporainDetails,
    ContemporainDocument,
    ItemEntry,
    Sheet,
  } from "../wasm/palimpseste_wasm";
  import {
    CHARACTERISTICS,
    U16_MAX,
    integerOf,
    namesOf,
    ranksOf,
    setRanks,
    signed,
    valueOf,
  } from "./edits";
  import LaterRanks from "../progression/LaterRanks.svelte";
  import { askingSpeciality } from "../progression/progression";
  import RankFivePicker from "../progression/RankFivePicker.svelte";
  import SpecialityField from "../progression/SpecialityField.svelte";
  import Items from "./Items.svelte";
  import RankPicker from "./RankPicker.svelte";

  let {
    choices = $bindable(),
    catalog,
    sheet,
    details,
    step,
    characterId,
  }: {
    choices: ContemporainDocument;
    catalog: ContemporainCatalog;
    sheet: Sheet;
    details: ContemporainDetails;
    step: string;
    characterId: number;
  } = $props();

  const voieNames = $derived(namesOf(catalog.voies));
  const domaine = $derived(catalog.domaines.find((d) => d.id === choices.domaine));
  const domaineVoies = $derived(new Set(domaine?.slots.flat() ?? []));

  function total(id: string): string {
    const value = sheet.values.find((line) => line.id === id)?.total;
    return value === undefined ? "?" : String(value);
  }

  function pointsOf(id: string): number | undefined {
    return choices.traits?.find((t) => t.id === id)?.points;
  }

  function setTrait(id: string, points: number | undefined) {
    choices.traits ??= [];
    const traits = choices.traits;
    const index = traits.findIndex((t) => t.id === id);
    if (points === undefined) {
      if (index >= 0) traits.splice(index, 1);
    } else if (index >= 0) {
      traits[index].points = points;
    } else {
      traits.push({ id, points });
    }
  }

  function setPoints(c: Characteristic, points: number | undefined) {
    choices.caracPoints ??= {};
    choices.caracPoints[c] = points ?? 0;
  }

  function setItems(items: ItemEntry[]) {
    choices.items = items;
  }

  function toggleProtection(id: string, worn: boolean) {
    const protections = (choices.protections ??= []);
    choices.protections = worn ? [...protections, id] : protections.filter((p) => p !== id);
  }

  const startingWeapons = $derived(
    catalog.voies.filter(
      (voie) => voie.startingWeapon && ranksOf(choices.voies, voie.id).length > 0,
    ),
  );
</script>

{#if step === "domaine-and-age"}
  <div class="flex flex-wrap items-end gap-4">
    <Label class="flex flex-col gap-2">
      Âge
      <div class="w-28">
        <Input
          type="number"
          min="0"
          step="1"
          value={choices.age ?? ""}
          onchange={(e) => {
            const age = integerOf(e, choices.age, { min: 0, max: U16_MAX });
            if (age !== null) (choices.age = age);
          }}
        />
      </div>
    </Label>
    <Helper class="pb-3 text-sm">Tranche d'âge : {details.ageBracket ?? "—"}</Helper>
  </div>
  <Label class="flex flex-col gap-2">
    Domaine
    <Select
      placeholder=""
      value={choices.domaine ?? ""}
      onchange={(e) => {
        const id = valueOf(e);
        (choices.domaine = id || undefined);
      }}
    >
      <option value="">— Choisir —</option>
      <optgroup label="Professionnels">
        {#each catalog.domaines.filter((d) => !d.extraProfessional) as d (d.id)}
          <option value={d.id}>{d.name}</option>
        {/each}
      </optgroup>
      <optgroup label="Extra-professionnels">
        {#each catalog.domaines.filter((d) => d.extraProfessional) as d (d.id)}
          <option value={d.id}>{d.name}</option>
        {/each}
      </optgroup>
    </Select>
  </Label>
  {#if domaine}
    <Helper class="text-sm">
      Retenez au moins {domaine.minimumRetained} des voies du domaine :
      {domaine.slots.map((slot) => slot.map((id) => voieNames.get(id)).join(" ou ")).join(", ")}.
    </Helper>
  {/if}
{:else if step === "voies"}
  <P weight="semibold">
    Points de capacité : {details.capacityPointsSpent} / {total("coc.capacity-points")} · Voies :
    {details.voieCount} / {total("coc.max-voies")} · Rang maximum : {total("coc.max-rank")}
  </P>
  <Listgroup class="flex-col">
    {#each [...catalog.voies].sort((a, b) => Number(domaineVoies.has(b.id)) - Number(domaineVoies.has(a.id))) as voie (voie.id)}
      <ListgroupItem class="flex-col items-start gap-2 text-gray-900 dark:text-white">
        <span class="flex items-center gap-2 font-semibold">
          {voie.name}
          {#if domaineVoies.has(voie.id)}<Badge>domaine</Badge>{/if}
        </span>
        <RankPicker
          {catalog}
          voie={voie.id}
          ranks={ranksOf(choices.voies, voie.id)}
          onchange={(ranks) => setRanks(choices, voie.id, ranks)}
        />
        <span class="inline-flex flex-wrap items-center gap-2 empty:hidden">
          <LaterRanks {catalog} held={details.abilitiesHeld} voie={voie.id} {characterId} />
        </span>
      </ListgroupItem>
    {/each}
  </Listgroup>
  {#each details.rankFive.filter((line) => line.progression == null && line.choice.length > 0) as line (line.voie)}
    <RankFivePicker bind:choices {catalog} {line} />
  {/each}
  {@const asking = askingSpeciality(catalog, details.abilitiesHeld, null)}
  {#if asking.length > 0}
    {#each asking as ability (ability)}
      <SpecialityField bind:choices {catalog} {ability} />
    {/each}
  {/if}
{:else if step === "traits"}
  <P weight="semibold">
    Avantages {signed(details.advantages)} · Désavantages {signed(details.disadvantages)} · Solde
    {signed(details.advantages + details.disadvantages)} · Classe sociale :
    {details.socialClass ?? "contradictoire"}
  </P>
  {#each [true, false] as advantage (advantage)}
    <Heading tag="h3" class="text-lg">{advantage ? "Avantages" : "Désavantages"}</Heading>
    <Listgroup class="flex-col">
      {#each catalog.traits.filter((t) => t.advantage === advantage) as t (t.id)}
        {@const points = pointsOf(t.id)}
        <ListgroupItem class="flex-wrap justify-between gap-2 text-gray-900 dark:text-white">
          <Checkbox
            checked={points !== undefined}
            onchange={(e) =>
              setTrait(
                t.id,
                e.currentTarget instanceof HTMLInputElement && e.currentTarget.checked
                  ? t.allowedPoints[0]
                  : undefined,
              )}
          >
            {t.name}
          </Checkbox>
          {#if points !== undefined}
            <Select
              placeholder=""
              size="sm"
              class="w-auto"
              aria-label="Points de {t.name}"
              value={String(points)}
              onchange={(e) => setTrait(t.id, Number(valueOf(e)))}
            >
              <optgroup label="Catalogue">
                {#each t.allowedPoints as p (p)}<option value={String(p)}>{p}</option>{/each}
              </optgroup>
              <optgroup label="Hors catalogue">
                {#each [1, 2, 3, 4, 5].filter((p) => !t.allowedPoints.includes(p)) as p (p)}
                  <option value={String(p)}>{p}</option>
                {/each}
              </optgroup>
            </Select>
          {/if}
        </ListgroupItem>
      {/each}
    </Listgroup>
  {/each}
{:else if step === "characteristics"}
  <P weight="semibold">
    Points de caractéristique : {details.caracPointsSpent} / {total("coc.carac-points")}
  </P>
  <Table class="w-auto">
    <TableBody>
      {#each CHARACTERISTICS as c (c)}
        <TableBodyRow>
          <TableHeadCell scope="row" class="py-2">{c}</TableHeadCell>
          <TableBodyCell class="py-2">
            <div class="w-24">
              <Input
                type="number"
                step="1"
                size="sm"
                aria-label="Points attribués à {c}"
                value={choices.caracPoints?.[c] ?? 0}
                onchange={(e) => {
                  const points = integerOf(e, choices.caracPoints?.[c] ?? 0);
                  if (points !== null) setPoints(c, points);
                }}
              />
            </div>
          </TableBodyCell>
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>
{:else if step === "equipment"}
  {#if startingWeapons.length > 0}
    <Heading tag="h3" class="text-lg">Équipement de départ des voies</Heading>
    <ul class="list-inside list-disc">
      {#each startingWeapons as voie (voie.id)}
        <li>{voie.name} : une arme ({voie.startingWeapon?.name})</li>
      {/each}
    </ul>
  {/if}
  <Heading tag="h3" class="text-lg">Protections</Heading>
  <div class="flex flex-col gap-2">
    {#each catalog.protections as protection (protection.id)}
      <Checkbox
        checked={choices.protections?.includes(protection.id) ?? false}
        onchange={(e) =>
          toggleProtection(
            protection.id,
            e.currentTarget instanceof HTMLInputElement && e.currentTarget.checked,
          )}
      >
        {protection.name}
      </Checkbox>
    {/each}
  </div>
  <Helper class="text-sm">Aucune voie ne donne de protection : en porter une se négocie avec le MJ.</Helper>
  <Heading tag="h3" class="text-lg">Objets portés</Heading>
  <Items
    items={choices.items ?? []}
    weapons={catalog.weapons.filter((w) => w.attackMode.id !== "unarmed")}
    onchange={setItems}
  />
{/if}
