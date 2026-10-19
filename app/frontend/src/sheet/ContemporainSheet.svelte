<script lang="ts">
  // A COC2 character at the table: the échelle de santé counted in the bar,
  // then what the player reads during a session, with the equipment in
  // play. Every number comes from the engine.
  import {
    Checkbox,
    Heading,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from "flowbite-svelte";

  import { namesOf, signed } from "../character/edits";
  import type {
    ContemporainCatalog,
    ContemporainDetails,
    ContemporainDocument,
    ContemporainEquipment,
    Sheet,
    ValueLine,
  } from "../wasm/palimpseste_wasm";
  import Block from "./Block.svelte";
  import Characteristics from "./Characteristics.svelte";
  import EquipmentSection from "./EquipmentSection.svelte";
  import HealthTrack from "./HealthTrack.svelte";
  import Marks from "./Marks.svelte";
  import NoteList from "./NoteList.svelte";
  import { type Note, marksAt, steps } from "./notes";
  import type { PlayState } from "./play.svelte";
  import Restore from "./Restore.svelte";
  import Tile from "./Tile.svelte";
  import { damage, range, shown, weaponSlugs } from "./values";
  import VoieRanks from "./VoieRanks.svelte";

  let {
    choices,
    catalog,
    sheet,
    details,
    play,
    notes,
    equipment,
  }: {
    choices: ContemporainDocument;
    catalog: ContemporainCatalog;
    sheet: Sheet;
    details: ContemporainDetails;
    play: PlayState;
    notes: Note[];
    /** The equipment in play, or the creation one. */
    equipment: ContemporainEquipment;
  } = $props();

  const HEALTH = "coc.health-scale";
  const ATTACKS = ["unarmed", "contact-weapon", "ranged-weapon", "heavy-weapon"];

  const domaine = $derived(catalog.domaines.find((d) => d.id === choices.domaine));
  const health = $derived(line(HEALTH)?.total);
  const identity = $derived(
    [
      choices.age === undefined
        ? "Âge à choisir"
        : `${choices.age} ans${details.ageBracket ? ` (${details.ageBracket})` : ""}`,
      domaine?.name ?? "Domaine à choisir",
      `Classe sociale : ${details.socialClass ?? "contradictoire"}`,
    ].join(" · "),
  );
  const protections = $derived(namesOf(catalog.protections));
  const traits = $derived(
    (choices.traits ?? []).flatMap((taken) => {
      const option = catalog.traits.find((t) => t.id === taken.id);
      return option ? [{ ...option, points: option.advantage ? taken.points : -taken.points }] : [];
    }),
  );

  function line(id: string): ValueLine | undefined {
    return sheet.values.find((l) => l.id === id);
  }

  function valueMarks(id: string): Note[] {
    return marksAt(notes, { kind: "value", id });
  }

  /** Wears and carries `change` in play, the rest of the equipment kept. */
  function equip(change: Partial<ContemporainEquipment>) {
    play.equip({ engine: "contemporain", ...equipment, ...change });
  }

  function wear(id: string, worn: boolean) {
    const others = (equipment.protections ?? []).filter((p) => p !== id);
    equip({ protections: worn ? [...others, id] : others });
  }

  const COMBAT = [
    { id: "coc.initiative", label: "Init", title: "Initiative", signed: false },
    { id: "coc.defence", label: "DEF", title: "Défense", signed: false },
    { id: "coc.damage-reduction", label: "RD", title: "Réduction des dommages", signed: false },
    {
      id: "coc.healing-capacity",
      label: "CG / sem.",
      title: "Capacité de guérison : DM soignés par semaine",
      signed: false,
    },
    { id: "coc.contact-damage-bonus", label: "DM contact", title: "Bonus aux DM au contact", signed: true },
  ];

  // Bare hands, then the weapons carried, from the engine's damage lines
  // (`coc.damage.<weapon>`), each rolled against its attack value.
  const weapons = $derived(
    weaponSlugs(sheet.values, "coc.damage").flatMap((slug) => {
      const weapon = catalog.weapons.find((w) => w.id === `coc.weapon.${slug}`);
      if (!weapon) return [];
      const quantity = (equipment.items ?? [])
        .filter((item) => item.kind === "weapon" && item.id === weapon.id)
        .reduce((sum, item) => sum + item.quantity, 0);
      return [
        {
          weapon,
          quantity,
          attack: line(`coc.attack.${weapon.attackMode.id}`),
          damage: line(`coc.damage.${slug}`),
          bonus: line(`coc.critical-bonus.${slug}`),
          range: line(`coc.critical-range.${slug}`),
        },
      ];
    }),
  );
</script>

<p class="mb-3 text-gray-500 dark:text-gray-400">
  {identity}<Marks notes={marksAt(notes, ...steps("domaine-and-age"))} />
</p>

<div
  class="sticky top-0 z-10 -mx-4 flex flex-wrap items-center gap-x-6 gap-y-3 border-b border-gray-200 bg-white px-4 py-2 print:static dark:border-gray-700 dark:bg-gray-900"
>
  {#if health !== undefined}
    <HealthTrack
      scale={catalog.healthScale}
      length={health}
      damage={play.used(HEALTH, health)}
      notes={valueMarks(HEALTH)}
      onchange={(damage) => play.set(HEALTH, damage, health)}
    />
  {/if}
  <Restore {play} />
</div>

<div class="mt-4 grid items-start gap-4 sm:grid-cols-2 lg:grid-cols-3">
  <Block title="Caractéristiques" notes={marksAt(notes, ...steps("characteristics"))}>
    <Characteristics characteristics={sheet.characteristics} {notes} />
  </Block>

  <Block title="Combat" notes={marksAt(notes, ...steps("secondary"))}>
    <ul class="grid grid-cols-[repeat(auto-fill,minmax(4rem,1fr))] gap-1.5">
      {#each COMBAT as tile (tile.id)}
        {@const value = line(tile.id)}
        <Tile
          label={tile.label}
          title={tile.title}
          value={shown(value, tile.signed)}
          notes={valueMarks(tile.id)}
          contributions={value?.contributions}
        />
      {/each}
      {#if line("coc.agi-test-modifier")?.total}
        {@const agi = line("coc.agi-test-modifier")}
        <Tile
          label="Tests d'AGI"
          value={shown(agi, true)}
          notes={valueMarks("coc.agi-test-modifier")}
          contributions={agi?.contributions}
        />
      {/if}
      {#each ATTACKS as mode (mode)}
        {@const attack = line(`coc.attack.${mode}`)}
        {#if attack}
          <Tile
            label={attack.label}
            value={shown(attack, true)}
            notes={valueMarks(attack.id)}
            contributions={attack.contributions}
          />
        {/if}
      {/each}
    </ul>
    <Table class="mt-3">
      <TableHead>
        <TableHeadCell class="px-2 py-2">Arme</TableHeadCell><TableHeadCell class="px-2 py-2">Att.</TableHeadCell><TableHeadCell class="px-2 py-2">DM</TableHeadCell><TableHeadCell class="px-2 py-2">Critique</TableHeadCell>
      </TableHead>
      <TableBody>
        {#each weapons as w (w.weapon.id)}
          <TableBodyRow>
            <TableHeadCell scope="row" class="px-2 py-2 whitespace-normal text-gray-900 dark:text-white">{w.weapon.name}{#if w.quantity > 1}{` ×${w.quantity}`}{/if}</TableHeadCell>
            <TableBodyCell title={w.attack?.label} class="px-2 py-2">{shown(w.attack, true)}</TableBodyCell>
            <TableBodyCell class="px-2 py-2">
              {damage(w.damage)}{#if w.damage}<Marks notes={valueMarks(w.damage.id)} />{/if}
            </TableBodyCell>
            <TableBodyCell class="px-2 py-2">
              {range(w.range)}{#if w.bonus?.total !== undefined}{` · ${signed(w.bonus.total)}`}{/if}
            </TableBodyCell>
          </TableBodyRow>
        {/each}
      </TableBody>
    </Table>
  </Block>

  <Block title="Voies" notes={marksAt(notes, ...steps("voies"))}>
    <VoieRanks voies={choices.voies ?? []} names={namesOf(catalog.voies)} {notes} />
  </Block>

  <Block title="Traits distinctifs" notes={marksAt(notes, ...steps("traits"))}>
    {#each [true, false] as advantage (advantage)}
      {@const listed = traits.filter((t) => t.advantage === advantage)}
      <Heading tag="h4" class="text-base text-gray-500 dark:text-gray-400">
        {advantage ? "Avantages" : "Désavantages"}
        {signed(advantage ? details.advantages : details.disadvantages)}
      </Heading>
      {#if listed.length > 0}
        <ul class="space-y-1">
          {#each listed as t (t.id)}
            <li>{t.name} ({signed(t.points)})</li>
          {/each}
        </ul>
      {:else}
        <p class="text-sm text-gray-500 dark:text-gray-400">Aucun.</p>
      {/if}
    {/each}
  </Block>

  <EquipmentSection
    notes={marksAt(notes, ...steps("equipment"))}
    items={equipment.items ?? []}
    weapons={catalog.weapons.filter((w) => w.attackMode.id !== "unarmed")}
    findings={sheet.inPlay}
    kept={play.equipment !== null}
    onitems={(items) => equip({ items })}
    onforget={() => play.forgetEquipment()}
  >
    {#snippet worn(editing: boolean)}
      {#if editing}
        <fieldset class="flex flex-col gap-2">
          <legend class="mb-2 text-sm font-medium text-gray-900 dark:text-white">Protections</legend>
          {#each catalog.protections as protection (protection.id)}
            <Checkbox
              checked={equipment.protections?.includes(protection.id) ?? false}
              onchange={(e) =>
                wear(protection.id, e.currentTarget instanceof HTMLInputElement && e.currentTarget.checked)}
            >
              {protection.name}
            </Checkbox>
          {/each}
        </fieldset>
      {:else}
        <li>
          Protections :
          {(equipment.protections ?? []).map((id) => protections.get(id) ?? id).join(", ") || "aucune"}
        </li>
      {/if}
    {/snippet}
  </EquipmentSection>

  <NoteList {notes} />
</div>
