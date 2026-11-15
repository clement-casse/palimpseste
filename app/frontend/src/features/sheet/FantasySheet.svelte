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

  import { namesOf, signed, valueOf } from "$lib/character/edits";
  import type {
    FantasyCatalog,
    FantasyDetails,
    FantasyDocument,
    FantasyEquipment,
    Sheet,
  } from "$lib/engine";
  import { COF } from "$lib/engine/ids";
  import Block from "./Block.svelte";
  import Characteristics from "./Characteristics.svelte";
  import Counter from "./Counter.svelte";
  import EquipmentSection from "./EquipmentSection.svelte";
  import Identity from "./Identity.svelte";
  import Marks from "$lib/components/Marks.svelte";
  import NoteList from "./NoteList.svelte";
  import { type Note, marksAt, steps } from "$lib/character/notes";
  import type { PlayState } from "./play.svelte";
  import Restore from "./Restore.svelte";
  import RollButton from "./RollButton.svelte";
  import SituationalList from "./SituationalList.svelte";
  import Tile from "./Tile.svelte";
  import { fantasySheetModel } from "./model/fantasy";
  import { damage, range, shown, situationalText } from "$lib/format";
  import VoieRanks from "./VoieRanks.svelte";

  let {
    choices,
    catalog,
    sheet,
    details,
    play,
    notes,
    equipment,
    onroll,
  }: {
    choices: FantasyDocument;
    catalog: FantasyCatalog;
    sheet: Sheet;
    details: FantasyDetails;
    /** What is kept on the device; none for a sheet only read. */
    play: PlayState | null;
    notes: Note[];
    /** Opens a roll by its identifier; none for a sheet only read. */
    onroll?: (roll: string) => void;
    /** The equipment in play, or the creation one. */
    equipment: FantasyEquipment;
  } = $props();

  const model = $derived(fantasySheetModel({ choices, catalog, sheet, details, equipment }));

  function valueMarks(id: string): Note[] {
    return marksAt(notes, { kind: "value", id });
  }

  /** Wears and carries `change` in play, the rest of the equipment kept. */
  function equip(change: Partial<FantasyEquipment>) {
    play?.equip({ engine: "fantasy", ...equipment, ...change });
  }
</script>

<div class="my-3">
  <Identity fields={model.identity} notes={marksAt(notes, ...steps("profil", "peuple"))} />
</div>

<div
  class="sticky top-0 z-10 -mx-4 flex flex-wrap items-center gap-x-6 gap-y-3 border-b border-gray-200 bg-white px-4 py-2 print:static dark:border-gray-700 dark:bg-gray-900"
>
  {#each model.resources as resource (resource.id)}
    <Counter
      label={resource.short}
      title={resource.label}
      max={resource.max}
      used={play?.used(resource.id, resource.max) ?? 0}
      die={resource.dice}
      notes={valueMarks(resource.id)}
      onchange={play ? (used) => play.set(resource.id, used, resource.max) : undefined}
    />
  {/each}
  {#if play}<Restore {play} />{/if}
</div>

<!-- As a printed sheet: the characteristics down the left, combat in the
     middle, the voies on the right, the equipment below. -->
<div class="mt-4 grid items-start gap-4 md:grid-cols-2 lg:grid-cols-12">
  <div class="grid gap-4 lg:col-span-3">
    <Block title="Caractéristiques" notes={marksAt(notes, ...steps("characteristics"))}>
      <Characteristics characteristics={sheet.characteristics} {notes} {catalog} {onroll} />
    </Block>
  </div>

  <div class="grid gap-4 lg:col-span-5">

    <Block title="Combat" notes={marksAt(notes, ...steps("derived"))}>
      <ul class="grid grid-cols-[repeat(auto-fill,minmax(4rem,1fr))] gap-1.5">
        {#each model.combat as tile (tile.id)}
          {@const roll = tile.roll}
          <Tile
            label={tile.label}
            title={tile.title}
            value={tile.text}
            notes={valueMarks(tile.id)}
            contributions={tile.contributions}
            situational={tile.situational}
            onroll={onroll && roll ? () => onroll(roll) : undefined}
          />
        {/each}
      </ul>
      <!-- Weapons show theirs in their row. -->
      <SituationalList {catalog} entries={model.situational} />
      {#if model.weapons.length > 0}
        <Table class="mt-3">
          <TableHead>
            <TableHeadCell class="px-2 py-2">Arme</TableHeadCell><TableHeadCell class="px-2 py-2">Usage</TableHeadCell><TableHeadCell class="px-2 py-2">Attaque</TableHeadCell><TableHeadCell class="px-2 py-2">DM</TableHeadCell><TableHeadCell class="px-2 py-2">Critique</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each model.weapons as { weapon, quantity, uses, critical } (weapon.id)}
              {#each uses as use, i (use.damage.id)}
                {@const dm = use.damage}
                <TableBodyRow>
                  {#if i === 0}
                    <TableHeadCell scope="row" rowspan={uses.length} class="px-2 py-2 whitespace-normal text-gray-900 dark:text-white">
                      {weapon.name}{#if quantity > 1}{` ×${quantity}`}{/if}
                    </TableHeadCell>
                  {/if}
                  <TableBodyCell class="px-2 py-2">{use.name}</TableBodyCell>
                  {@const attackId = use.attackId}
                  <TableBodyCell class="px-2 py-2">{shown(use.attack, true)}<Marks notes={valueMarks(attackId)} />{#if onroll && dm.attackRoll}{@const attackRoll = dm.attackRoll} <RollButton label="attaque, {weapon.name} ({use.name})" onclick={() => onroll(attackRoll)} />{/if}{#each use.attack?.situational ?? [] as s, k (k)}<span class="block text-xs text-gray-500 dark:text-gray-400">{situationalText(s)}</span>{/each}</TableBodyCell>
                  <TableBodyCell class="px-2 py-2">{damage(dm)}<Marks notes={valueMarks(dm.id)} />{#if onroll && dm.roll}{@const dmRoll = dm.roll} <RollButton label="DM, {weapon.name} ({use.name})" onclick={() => onroll(dmRoll)} />{/if}{#each dm.situational ?? [] as s, k (k)}<span class="block text-xs text-gray-500 dark:text-gray-400">{situationalText(s)}</span>{/each}</TableBodyCell>
                  {#if i === 0}
                    <TableBodyCell rowspan={uses.length} class="px-2 py-2">
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
  </div>

  <div class="grid gap-4 md:col-span-2 lg:col-span-4">
    <Block title="Voies" notes={marksAt(notes, ...steps("voies"))}>
      <VoieRanks
        voies={model.voies}
        names={namesOf(catalog.voies)}
        {notes}
        {catalog}
        held={details.abilitiesHeld}
        written={sheet.writtenEffects}
        levels
      />
    </Block>
  </div>

  <div class="grid items-start gap-4 md:col-span-2 md:grid-cols-2 lg:col-span-12">

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
          {@const { armour, shield } = model}
          <li>Armure : {armour ? `${armour.name} (DEF ${signed(armour.defence)})` : "aucune"}</li>
          <li>Bouclier : {shield ? `${shield.name} (DEF ${signed(shield.defence)})` : "aucun"}</li>
          <li>Bourse : {model.money ?? "—"}<Marks notes={valueMarks(COF.money)} /></li>
        {/if}
      {/snippet}
    </EquipmentSection>

    <NoteList {notes} />
  </div>
</div>
