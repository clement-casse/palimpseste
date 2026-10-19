<script lang="ts">
  // A COF2 character at the table: PV, PM, PC and DR counted in the bar,
  // then what the player reads during a session, with the equipment in
  // play. Every number comes from the engine.
  import {
    Label,
    Select,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from "flowbite-svelte";

  import { namesOf, signed, valueOf } from "../character/edits";
  import type {
    FantasyCatalog,
    FantasyDetails,
    FantasyDocument,
    FantasyEquipment,
    Sheet,
    ValueLine,
  } from "../wasm/palimpseste_wasm";
  import Block from "./Block.svelte";
  import Characteristics from "./Characteristics.svelte";
  import Counter from "./Counter.svelte";
  import EquipmentSection from "./EquipmentSection.svelte";
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
    choices: FantasyDocument;
    catalog: FantasyCatalog;
    sheet: Sheet;
    details: FantasyDetails;
    /** What is kept on the device; none for a sheet only read. */
    play: PlayState | null;
    notes: Note[];
    /** The equipment in play, or the creation one. */
    equipment: FantasyEquipment;
  } = $props();

  const profil = $derived(catalog.profils.find((p) => p.id === choices.profil));
  const peuple = $derived(catalog.peuples.find((p) => p.id === choices.peuple));
  const armour = $derived(catalog.armours.find((a) => a.id === equipment.armour));
  const shield = $derived(catalog.shields.find((s) => s.id === equipment.shield));
  const identity = $derived(
    [
      profil ? `${profil.name}${details.family ? ` (${details.family.name})` : ""}` : "Profil à choisir",
      peuple?.name ?? "Peuple à choisir",
      "Niveau 1",
    ].join(" · "),
  );

  function line(id: string): ValueLine | undefined {
    return sheet.values.find((l) => l.id === id);
  }

  function valueMarks(id: string): Note[] {
    return marksAt(notes, { kind: "value", id });
  }

  /** Wears and carries `change` in play, the rest of the equipment kept. */
  function equip(change: Partial<FantasyEquipment>) {
    play?.equip({ engine: "fantasy", ...equipment, ...change });
  }

  const COMBAT = [
    { id: "cof.initiative", label: "Init", title: "Initiative", signed: false },
    { id: "cof.defence", label: "DEF", title: "Défense", signed: false },
    { id: "cof.attack.contact", label: "Contact", title: "Attaque au contact", signed: true },
    { id: "cof.attack.ranged", label: "Distance", title: "Attaque à distance", signed: true },
    { id: "cof.attack.magic", label: "Magique", title: "Attaque magique", signed: true },
  ];

  // The resources, each counted against its computed maximum; PM only for a
  // character who knows a spell (COF.DER.PM).
  const resources = $derived(
    [
      { id: "cof.vigour-points", label: "PV" },
      { id: "cof.mana-points", label: "PM" },
      { id: "cof.luck-points", label: "PC" },
      { id: "cof.recovery-dice", label: "DR" },
    ].flatMap(({ id, label }) => {
      const value = line(id);
      return value?.total === undefined ? [] : [{ ...value, total: value.total, short: label }];
    }),
  );

  // The weapons carried, in the document's order, from the engine's damage
  // lines (`cof.damage.<weapon>.<use>`): one per use, in the order of the
  // catalogue's uses, then the critical range.
  const weapons = $derived(
    weaponSlugs(sheet.values, "cof.damage").flatMap((slug) => {
      const weapon = catalog.weapons.find((w) => w.id === `cof.weapon.${slug}`);
      if (!weapon) return [];
      const damages = sheet.values.filter((l) => l.id.startsWith(`cof.damage.${slug}.`));
      const quantity = (equipment.items ?? [])
        .filter((item) => item.kind === "weapon" && item.id === weapon.id)
        .reduce((sum, item) => sum + item.quantity, 0);
      const critical = line(`cof.critical-range.${slug}`);
      return [{ weapon, quantity, damages, critical }];
    }),
  );
</script>

<p class="mb-3 text-gray-500 dark:text-gray-400">
  {identity}<Marks notes={marksAt(notes, ...steps("profil", "peuple"))} />
</p>

<div
  class="sticky top-0 z-10 -mx-4 flex flex-wrap items-center gap-x-6 gap-y-3 border-b border-gray-200 bg-white px-4 py-2 print:static dark:border-gray-700 dark:bg-gray-900"
>
  {#each resources as resource (resource.id)}
    <Counter
      label={resource.short}
      title={resource.label}
      max={resource.total}
      used={play?.used(resource.id, resource.total) ?? 0}
      die={resource.dice}
      notes={valueMarks(resource.id)}
      onchange={play ? (used) => play.set(resource.id, used, resource.total) : undefined}
    />
  {/each}
  {#if play}<Restore {play} />{/if}
</div>

<div class="mt-4 grid items-start gap-4 sm:grid-cols-2 lg:grid-cols-3">
  <Block title="Caractéristiques" notes={marksAt(notes, ...steps("characteristics"))}>
    <Characteristics characteristics={sheet.characteristics} {notes} />
  </Block>

  <Block title="Combat" notes={marksAt(notes, ...steps("derived"))}>
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
    </ul>
    {#if weapons.length > 0}
      <Table class="mt-3">
        <TableHead>
          <TableHeadCell class="px-2 py-2">Arme</TableHeadCell><TableHeadCell class="px-2 py-2">Usage</TableHeadCell><TableHeadCell class="px-2 py-2">DM</TableHeadCell><TableHeadCell class="px-2 py-2">Critique</TableHeadCell>
        </TableHead>
        <TableBody>
          {#each weapons as { weapon, quantity, damages, critical } (weapon.id)}
            {#each damages as dm, i (dm.id)}
              <TableBodyRow>
                {#if i === 0}
                  <TableHeadCell scope="row" rowspan={damages.length} class="px-2 py-2 whitespace-normal text-gray-900 dark:text-white">
                    {weapon.name}{#if quantity > 1}{` ×${quantity}`}{/if}
                  </TableHeadCell>
                {/if}
                <TableBodyCell class="px-2 py-2">{weapon.uses[i]?.name ?? ""}</TableBodyCell>
                <TableBodyCell class="px-2 py-2">{damage(dm)}<Marks notes={valueMarks(dm.id)} /></TableBodyCell>
                {#if i === 0}
                  <TableBodyCell rowspan={damages.length} class="px-2 py-2">
                    {range(critical)}{#if critical}<Marks notes={valueMarks(critical.id)} />{/if}
                  </TableBodyCell>
                {/if}
              </TableBodyRow>
            {/each}
          {/each}
        </TableBody>
      </Table>
    {/if}
  </Block>

  <Block title="Voies" notes={marksAt(notes, ...steps("voies"))}>
    <VoieRanks voies={choices.voies ?? []} names={namesOf(catalog.voies)} {notes} />
  </Block>

  <EquipmentSection
    notes={marksAt(notes, ...steps("equipment"))}
    items={equipment.items ?? []}
    weapons={catalog.weapons}
    findings={sheet.inPlay}
    kept={play !== null && play.equipment !== null}
    onitems={play ? (items) => equip({ items }) : undefined}
    onforget={() => play?.forgetEquipment()}
  >
    {#snippet worn(editing: boolean)}
      {#if editing}
        <Label class="flex flex-col gap-2">
          Armure
          <Select
            placeholder=""
            value={equipment.armour ?? ""}
            onchange={(e) => equip({ armour: valueOf(e) || undefined })}
          >
            <option value="">Aucune</option>
            {#each catalog.armours as a (a.id)}
              <option value={a.id}>{a.name} (DEF {signed(a.defence)})</option>
            {/each}
          </Select>
        </Label>
        <Label class="flex flex-col gap-2">
          Bouclier
          <Select
            placeholder=""
            value={equipment.shield ?? ""}
            onchange={(e) => equip({ shield: valueOf(e) || undefined })}
          >
            <option value="">Aucun</option>
            {#each catalog.shields as s (s.id)}
              <option value={s.id}>{s.name} (DEF {signed(s.defence)})</option>
            {/each}
          </Select>
        </Label>
      {:else}
        <li>Armure : {armour ? `${armour.name} (DEF ${signed(armour.defence)})` : "aucune"}</li>
        <li>Bouclier : {shield ? `${shield.name} (DEF ${signed(shield.defence)})` : "aucun"}</li>
        <li>Bourse : {details.money ?? "—"}<Marks notes={valueMarks("cof.money")} /></li>
      {/if}
    {/snippet}
  </EquipmentSection>

  <NoteList {notes} />
</div>
