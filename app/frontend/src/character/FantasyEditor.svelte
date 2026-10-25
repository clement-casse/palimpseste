<script lang="ts">
  // The choices of a COF2 character, one section per creation step. The
  // options the rules allow come first; anything else stays reachable under
  // "Hors règles", and the engine reports it.
  import {
    Accordion,
    AccordionItem,
    Button,
    DescriptionList,
    Heading,
    Helper,
    Input,
    Label,
    Listgroup,
    ListgroupItem,
    Radio,
    Select,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHeadCell,
  } from "flowbite-svelte";

  import type {
    Characteristic,
    FantasyCatalog,
    FantasyDetails,
    FantasyDocument,
    ItemEntry,
  } from "../wasm/palimpseste_wasm";
  import {
    CHARACTERISTICS,
    I32_MAX,
    I32_MIN,
    integerOf,
    namesOf,
    ranksOf,
    setRanks,
    signed,
    valueOf,
  } from "./edits";
  import AbilityOptionPicker from "../progression/AbilityOptionPicker.svelte";
  import { askingSpeciality } from "../progression/progression";
  import SpecialityField from "../progression/SpecialityField.svelte";
  import LaterRanks from "../progression/LaterRanks.svelte";
  import Items from "./Items.svelte";
  import RankPicker from "./RankPicker.svelte";

  let {
    choices = $bindable(),
    catalog,
    details,
    step,
    characterId,
  }: {
    choices: FantasyDocument;
    catalog: FantasyCatalog;
    details: FantasyDetails;
    step: string;
    characterId: number;
  } = $props();

  const voieNames = $derived(namesOf(catalog.voies));
  const gearNames = $derived(
    namesOf([...catalog.weapons, ...catalog.armours, ...catalog.shields]),
  );
  const profil = $derived(catalog.profils.find((p) => p.id === choices.profil));
  const peuple = $derived(catalog.peuples.find((p) => p.id === choices.peuple));
  const series = $derived(catalog.series.find((s) => s.id === choices.series));

  // Characteristics switched to a value outside the series.
  let custom = $state<Partial<Record<Characteristic, boolean>>>({});

  function assigned(c: Characteristic): number | undefined {
    return choices.seriesValues?.[c] ?? undefined;
  }

  function seriesChoice(c: Characteristic): string {
    const value = assigned(c);
    if (custom[c] || (value !== undefined && !series?.values.includes(value))) return "other";
    return value === undefined ? "" : String(value);
  }

  /** How many times `value` is still free in the series, besides `c`. */
  function free(value: number, c: Characteristic): number {
    const total = series?.values.filter((v) => v === value).length ?? 0;
    const used = CHARACTERISTICS.filter((o) => o !== c && assigned(o) === value).length;
    return total - used;
  }

  function assign(c: Characteristic, choice: string) {
    custom[c] = choice === "other";
    if (choice === "other") return;
    choices.seriesValues ??= {};
    choices.seriesValues[c] = choice === "" ? undefined : Number(choice);
  }

  function assignOther(c: Characteristic, value: number | undefined) {
    choices.seriesValues ??= {};
    choices.seriesValues[c] = value;
  }

  function keyMark(c: Characteristic): string {
    const key = profil?.keyCharacteristics.some((slot) => slot.includes(c));
    const magic = profil?.magicCharacteristic === c;
    return [key ? "clé" : "", magic ? "magie" : ""].filter(Boolean).join(", ");
  }

  const otherVoies = $derived(
    (choices.voies ?? [])
      .map((voie) => voie.id)
      .filter((id) => !profil?.voies.includes(id) && !details.peupleSlotOptions.includes(id)),
  );

  function setVoie(id: string, ranks: number[]) {
    setRanks(choices, id, ranks);
  }

  /** Whether an armour or shield is within the profil's limit; without a profil, every one is. */
  function within(list: { id: string }[], id: string, limit: string | undefined): boolean {
    if (!profil) return true;
    if (limit === undefined) return false;
    return list.findIndex((x) => x.id === id) <= list.findIndex((x) => x.id === limit);
  }

  function addGear(id: string) {
    if (catalog.armours.some((a) => a.id === id)) choices.armour = id;
    else if (catalog.shields.some((s) => s.id === id)) choices.shield = id;
    else {
      choices.items ??= [];
      const items = choices.items;
      const item = items.find((i) => i.kind === "weapon" && i.id === id);
      if (item) item.quantity += 1;
      else items.push({ kind: "weapon", id, quantity: 1 });
    }
  }

  function setItems(items: ItemEntry[]) {
    choices.items = items;
  }
</script>

{#if step === "profil"}
  <Label class="flex flex-col gap-2">
    Profil
    <Select
      placeholder=""
      value={choices.profil ?? ""}
      onchange={(e) => {
        const id = valueOf(e);
        (choices.profil = id || undefined);
      }}
    >
      <option value="">— Choisir —</option>
      {#each catalog.families as family (family.id)}
        <optgroup label={family.name}>
          {#each catalog.profils.filter((p) => p.family === family.id) as p (p.id)}
            <option value={p.id}>{p.name}</option>
          {/each}
        </optgroup>
      {/each}
    </Select>
  </Label>
  {#if profil}
    <dl class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1">
      <DescriptionList tag="dt" class="md:text-base">Famille</DescriptionList>
      <DescriptionList tag="dd" class="text-base">{details.family?.name}</DescriptionList>
      <DescriptionList tag="dt" class="md:text-base">Voies</DescriptionList>
      <DescriptionList tag="dd" class="text-base">{profil.voies.map((id) => voieNames.get(id)).join(", ")}</DescriptionList>
      <DescriptionList tag="dt" class="md:text-base">Caractéristiques clés</DescriptionList>
      <DescriptionList tag="dd" class="text-base">{profil.keyCharacteristics.map((slot) => slot.join(" ou ")).join(", ")}</DescriptionList>
      <DescriptionList tag="dt" class="md:text-base">Caractéristique de magie</DescriptionList>
      <DescriptionList tag="dd" class="text-base">{profil.magicCharacteristic ?? "aucune"}</DescriptionList>
      <DescriptionList tag="dt" class="md:text-base">Armure</DescriptionList>
      <DescriptionList tag="dd" class="text-base">{profil.armourLimit ? `jusqu'à ${gearNames.get(profil.armourLimit)}` : "aucune"}</DescriptionList>
      <DescriptionList tag="dt" class="md:text-base">Bouclier</DescriptionList>
      <DescriptionList tag="dd" class="text-base">{profil.shieldLimit ? `jusqu'au ${gearNames.get(profil.shieldLimit)}` : "aucun"}</DescriptionList>
      <DescriptionList tag="dt" class="md:text-base">Armes maîtrisées</DescriptionList>
      <DescriptionList tag="dd" class="text-base">{profil.masteredWeapons.map((id) => gearNames.get(id)).join(", ")}</DescriptionList>
    </dl>
  {/if}
{:else if step === "peuple"}
  <Label class="flex flex-col gap-2">
    Peuple
    <Select
      placeholder=""
      value={choices.peuple ?? ""}
      onchange={(e) => {
        const id = valueOf(e);
        (choices.peuple = id || undefined);
      }}
    >
      <option value="">— Choisir —</option>
      {#each catalog.peuples as p (p.id)}
        <option value={p.id}>{p.name}</option>
      {/each}
    </Select>
  </Label>
  {#if peuple}
    <dl class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1">
      <DescriptionList tag="dt" class="md:text-base">Bonus (+1)</DescriptionList>
      <DescriptionList tag="dd" class="text-base">
        {peuple.bonusOptions.length === 7
          ? "une des deux caractéristiques les plus basses"
          : peuple.bonusOptions.join(" ou ")}
      </DescriptionList>
      <DescriptionList tag="dt" class="md:text-base">Malus (-1)</DescriptionList>
      <DescriptionList tag="dd" class="text-base">{peuple.malusOptions.length ? peuple.malusOptions.join(" ou ") : "aucun"}</DescriptionList>
      <DescriptionList tag="dt" class="md:text-base">Voie de peuple</DescriptionList>
      <DescriptionList tag="dd" class="text-base">
        {peuple.voie
          ? voieNames.get(peuple.voie)
          : "celle de l'humain ou d'un elfe, selon l'éducation"}
      </DescriptionList>
    </dl>
  {/if}
{:else if step === "characteristics"}
  <fieldset class="flex flex-col gap-2">
    <legend class="mb-2 text-sm font-medium text-gray-900 dark:text-white">Série de valeurs</legend>
    {#each catalog.series as s (s.id)}
      <Radio name="series" checked={choices.series === s.id} onchange={() => (choices.series = s.id)}>
        {s.name} ({s.values.map(signed).join(", ")})
      </Radio>
    {/each}
  </fieldset>
  <Table class="w-auto">
    <TableBody>
      {#each CHARACTERISTICS as c (c)}
        <TableBodyRow>
          <TableHeadCell scope="row" class="py-2">
            {c} <small class="font-normal text-gray-500 normal-case dark:text-gray-400">{keyMark(c)}</small>
          </TableHeadCell>
          <TableBodyCell class="flex flex-wrap gap-2 py-2">
            <Select
              placeholder=""
              size="sm"
              class="w-auto"
              aria-label="Valeur de {c}"
              value={seriesChoice(c)}
              onchange={(e) => assign(c, valueOf(e))}
            >
              <option value="">—</option>
              {#each [...new Set(series?.values ?? [])] as value (value)}
                <option value={String(value)}>
                  {signed(value)}{free(value, c) <= 0 ? " (déjà attribuée)" : ""}
                </option>
              {/each}
              <option value="other">Hors série…</option>
            </Select>
            {#if seriesChoice(c) === "other"}
              <div class="w-24">
                <Input
                  type="number"
                  step="1"
                  size="sm"
                  aria-label="Valeur hors série de {c}"
                  value={assigned(c) ?? ""}
                  onchange={(e) => {
                    const value = integerOf(e, assigned(c));
                    if (value !== null) assignOther(c, value);
                  }}
                />
              </div>
            {/if}
          </TableBodyCell>
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>
  {#if peuple}
    <Label class="flex flex-col gap-2">
      Bonus de peuple (+1)
      <Select
        placeholder=""
        value={choices.peupleBonus ?? ""}
        onchange={(e) => {
          const c = valueOf(e) as Characteristic | "";
          (choices.peupleBonus = c || undefined);
        }}
      >
        <option value="">— Choisir —</option>
        <optgroup label="Offert par le peuple">
          {#each peuple.bonusOptions as c (c)}<option value={c}>{c}</option>{/each}
        </optgroup>
        <optgroup label="Hors règles">
          {#each CHARACTERISTICS.filter((c) => !peuple.bonusOptions.includes(c)) as c (c)}
            <option value={c}>{c}</option>
          {/each}
        </optgroup>
      </Select>
    </Label>
    <Label class="flex flex-col gap-2">
      Malus de peuple (-1)
      <Select
        placeholder=""
        value={choices.peupleMalus ?? ""}
        onchange={(e) => {
          const c = valueOf(e) as Characteristic | "";
          (choices.peupleMalus = c || undefined);
        }}
      >
        <option value="">
          {peuple.malusOptions.length === 1
            ? `${peuple.malusOptions[0]} (imposé par le peuple)`
            : peuple.malusOptions.length === 0
              ? "Aucun"
              : "— Choisir —"}
        </option>
        {#if peuple.malusOptions.length > 1}
          <optgroup label="Offert par le peuple">
            {#each peuple.malusOptions as c (c)}<option value={c}>{c}</option>{/each}
          </optgroup>
        {/if}
        <optgroup label="Hors règles">
          {#each CHARACTERISTICS.filter((c) => peuple.malusOptions.length === 1 || !peuple.malusOptions.includes(c)) as c (c)}
            <option value={c}>{c}</option>
          {/each}
        </optgroup>
      </Select>
    </Label>
  {/if}
{:else if step === "voies"}
  {#snippet voieRow(id: string)}
    <ListgroupItem class="flex-wrap justify-between gap-2 text-gray-900 dark:text-white">
      <span>{voieNames.get(id) ?? id}</span>
      <span class="inline-flex flex-wrap items-center gap-2">
        <RankPicker ranks={ranksOf(choices.voies, id)} onchange={(r) => setVoie(id, r)} />
        <LaterRanks {catalog} held={details.abilitiesHeld} voie={id} {characterId} levels />
      </span>
    </ListgroupItem>
  {/snippet}
  <Helper class="text-sm">
    Deux voies du profil au rang 1, et la voie de peuple au rang 1. Un mage prend en plus un rang 2.
  </Helper>
  <Heading tag="h3" class="text-lg">Voies du profil</Heading>
  {#if profil}
    <Listgroup class="flex-col">
      {#each profil.voies as id (id)}
        {@render voieRow(id)}
      {/each}
    </Listgroup>
  {:else}
    <Helper class="text-sm">Choisissez d'abord un profil.</Helper>
  {/if}
  <Heading tag="h3" class="text-lg">Voie de peuple</Heading>
  {#if details.peupleSlotOptions.length > 0}
    <Listgroup class="flex-col">
      {#each details.peupleSlotOptions as id (id)}
        {@render voieRow(id)}
      {/each}
    </Listgroup>
  {:else}
    <Helper class="text-sm">Choisissez d'abord un peuple.</Helper>
  {/if}
  {@const creationOptions = details.abilityOptions.filter((line) => line.progression == null)}
  {#if creationOptions.length > 0}
    <Heading tag="h3" class="text-lg">Options des capacités</Heading>
    {#each creationOptions as line (line.ability)}
      <AbilityOptionPicker bind:choices {catalog} {line} />
    {/each}
  {/if}
  {@const asking = askingSpeciality(catalog, details.abilitiesHeld, null)}
  {#if asking.length > 0}
    <Heading tag="h3" class="text-lg">Spécialités des capacités</Heading>
    {#each asking as ability (ability)}
      <SpecialityField bind:choices {catalog} {ability} />
    {/each}
  {/if}
  <Accordion flush>
    <AccordionItem open={otherVoies.length > 0} headingTag="h3" classes={{ content: "space-y-3 py-3" }}>
      {#snippet header()}Hors règles : autres voies{/snippet}
    {#if otherVoies.length > 0}
      <Listgroup class="flex-col">
        {#each otherVoies as id (id)}
          {@render voieRow(id)}
        {/each}
      </Listgroup>
    {/if}
    <Select
      placeholder=""
      class="w-auto"
      aria-label="Ajouter une voie"
      onchange={(e) => {
        const id = valueOf(e);
        if (e.currentTarget instanceof HTMLSelectElement) e.currentTarget.value = "";
        if (id) setVoie(id, [1]);
      }}
    >
      <option value="">Ajouter une voie…</option>
      {#each catalog.voies.filter((v) => !ranksOf(choices.voies, v.id).length) as v (v.id)}
        <option value={v.id}>{v.name}</option>
      {/each}
    </Select>
    </AccordionItem>
  </Accordion>
{:else if step === "equipment"}
  <div class="flex flex-wrap gap-4">
    <Label class="flex flex-col gap-2">
      Armure
      <Select
        placeholder=""
        value={choices.armour ?? ""}
        onchange={(e) => {
          const id = valueOf(e);
          (choices.armour = id || undefined);
        }}
      >
        <option value="">Aucune</option>
        <optgroup label="Permise par le profil">
          {#each catalog.armours.filter((a) => within(catalog.armours, a.id, profil?.armourLimit)) as a (a.id)}
            <option value={a.id}>{a.name} (DEF +{a.defence})</option>
          {/each}
        </optgroup>
        <optgroup label="Hors règles">
          {#each catalog.armours.filter((a) => !within(catalog.armours, a.id, profil?.armourLimit)) as a (a.id)}
            <option value={a.id}>{a.name} (DEF +{a.defence})</option>
          {/each}
        </optgroup>
      </Select>
    </Label>
    <Label class="flex flex-col gap-2">
      Bouclier
      <Select
        placeholder=""
        value={choices.shield ?? ""}
        onchange={(e) => {
          const id = valueOf(e);
          (choices.shield = id || undefined);
        }}
      >
        <option value="">Aucun</option>
        <optgroup label="Permis par le profil">
          {#each catalog.shields.filter((s) => within(catalog.shields, s.id, profil?.shieldLimit)) as s (s.id)}
            <option value={s.id}>{s.name} (DEF +{s.defence})</option>
          {/each}
        </optgroup>
        <optgroup label="Hors règles">
          {#each catalog.shields.filter((s) => !within(catalog.shields, s.id, profil?.shieldLimit)) as s (s.id)}
            <option value={s.id}>{s.name} (DEF +{s.defence})</option>
          {/each}
        </optgroup>
      </Select>
    </Label>
  </div>
  {#if profil}
    <Heading tag="h3" class="text-lg">Équipement de départ du profil</Heading>
    <ul class="space-y-2">
      {#each profil.startingEquipment as line, i (i)}
        <li class="flex flex-wrap items-center gap-2">
          {#each line as bundle, j (j)}
            {#if j > 0}<em class="text-gray-500 dark:text-gray-400">ou</em>{/if}
            <span class="inline-flex flex-wrap items-center gap-x-3 gap-y-1">
              {#each bundle as item, k (k)}
                <span class="inline-flex items-center gap-1">
                  {#if item.quantity > 1}{item.quantity} ×{/if}
                  {#each item.options as id, m (id)}
                    {#if m > 0}/{/if}
                    <Button size="xs" color="alternative" onclick={() => addGear(id)}>
                      {gearNames.get(id)}
                    </Button>
                  {/each}
                </span>
              {/each}
            </span>
          {/each}
        </li>
      {/each}
    </ul>
    <Helper class="text-sm">
      Plus le sac d'aventurier. Un objet spécial se négocie avec le MJ, contre un objet de valeur
      proche.
    </Helper>
  {/if}
  <Heading tag="h3" class="text-lg">Objets portés</Heading>
  <Items items={choices.items ?? []} weapons={catalog.weapons} onchange={setItems} />
  <Label class="flex flex-col gap-2">
    Bourse ({catalog.purseDice} pa, à lancer)
    <div class="w-32">
      <Input
        type="number"
        step="1"
        value={choices.purse === undefined ? "" : choices.purse / 10}
        onchange={(e) => {
          // Typed in pa, held in pc.
          const pa = integerOf(e, choices.purse === undefined ? undefined : choices.purse / 10, {
            min: Math.ceil(I32_MIN / 10),
            max: Math.floor(I32_MAX / 10),
          });
          if (pa !== null) (choices.purse = pa === undefined ? undefined : pa * 10);
        }}
      />
    </div>
  </Label>
  {#if details.money}<Helper class="text-sm">Argent : {details.money}</Helper>{/if}
{/if}
