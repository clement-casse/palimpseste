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

  import { namesOf, signed } from "$lib/character/edits";
  import type {
    ContemporainCatalog,
    ContemporainDetails,
    ContemporainDocument,
    ContemporainEquipment,
    Sheet,
  } from "$lib/engine";
  import { COC } from "$lib/engine/ids";
  import Block from "./Block.svelte";
  import Characteristics from "./Characteristics.svelte";
  import EquipmentSection from "./EquipmentSection.svelte";
  import HealthTrack from "./HealthTrack.svelte";
  import Identity from "./Identity.svelte";
  import Marks from "$lib/components/Marks.svelte";
  import NoteList from "./NoteList.svelte";
  import { type Note, marksAt, steps } from "$lib/character/notes";
  import type { PlayState } from "./play.svelte";
  import Restore from "./Restore.svelte";
  import RollButton from "./RollButton.svelte";
  import SituationalList from "./SituationalList.svelte";
  import Tile from "./Tile.svelte";
  import { contemporainSheetModel } from "./model/contemporain";
  import { damage, range, shown, situationalText } from "$lib/format";
  import { characterPath, follow } from "$lib/router.svelte";
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
    characterId,
  }: {
    /** The user's own character, to record what play brings; none for a friend's. */
    characterId?: number;
    choices: ContemporainDocument;
    catalog: ContemporainCatalog;
    sheet: Sheet;
    details: ContemporainDetails;
    /** What is kept on the device; none for a sheet only read. */
    play: PlayState | null;
    notes: Note[];
    /** Opens a roll by its identifier; none for a sheet only read. */
    onroll?: (roll: string) => void;
    /** The equipment in play, or the creation one. */
    equipment: ContemporainEquipment;
  } = $props();

  const HEALTH = COC.healthScale;

  const model = $derived(contemporainSheetModel({ choices, catalog, sheet, details, equipment }));
  const health = $derived(model.health);

  function valueMarks(id: string): Note[] {
    return marksAt(notes, { kind: "value", id });
  }

  /** Wears and carries `change` in play, the rest of the equipment kept. */
  function equip(change: Partial<ContemporainEquipment>) {
    play?.equip({ engine: "contemporain", ...equipment, ...change });
  }

  function wear(id: string, worn: boolean) {
    const others = (equipment.protections ?? []).filter((p) => p !== id);
    equip({ protections: worn ? [...others, id] : others });
  }
</script>

<div class="my-3">
  <Identity fields={model.identity} notes={marksAt(notes, ...steps("domaine-and-age"))} />
</div>

<div
  class="sticky top-0 z-10 -mx-4 flex flex-wrap items-center gap-x-6 gap-y-3 border-b border-gray-200 bg-white px-4 py-2 print:static dark:border-gray-700 dark:bg-gray-900"
>
  {#if health !== undefined}
    <HealthTrack
      scale={catalog.healthScale}
      length={health}
      damage={play?.used(HEALTH, health) ?? 0}
      reached={details.healthStates}
      notes={valueMarks(HEALTH)}
      onchange={play ? (damage) => play.set(HEALTH, damage, health) : undefined}
    />
    {#if play && characterId !== undefined && play.used(HEALTH, health) >= health}
      <a
        class="text-sm underline"
        href={`${characterPath(characterId)}#progression`}
        onclick={follow}
      >
        Séquelle subie ? L’inscrire
      </a>
    {/if}
  {/if}
  {#if play}<Restore {play} />{/if}
</div>

<!-- As a printed sheet: the characteristics down the left,
     combat in the middle, the voies and traits on the right, the equipment below. -->
<div class="mt-4 grid items-start gap-4 md:grid-cols-2 lg:grid-cols-12">
  <div class="grid gap-4 lg:col-span-3">
    <Block title="Caractéristiques" notes={marksAt(notes, ...steps("characteristics"))}>
      <Characteristics characteristics={sheet.characteristics} {notes} {catalog} {onroll} />
    </Block>
  </div>

  <div class="grid gap-4 lg:col-span-5">
    <Block title="Combat" notes={marksAt(notes, ...steps("secondary"))}>
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
      <Table class="mt-3">
        <TableHead>
          <TableHeadCell class="px-2 py-2">Arme</TableHeadCell><TableHeadCell class="px-2 py-2"
            >Att.</TableHeadCell
          ><TableHeadCell class="px-2 py-2">DM</TableHeadCell><TableHeadCell class="px-2 py-2"
            >Critique</TableHeadCell
          >
        </TableHead>
        <TableBody>
          {#each model.weapons as w (w.weapon.id)}
            <TableBodyRow>
              <TableHeadCell
                scope="row"
                class="px-2 py-2 whitespace-normal text-gray-900 dark:text-white"
                >{w.weapon.name}{#if w.quantity > 1}{` ×${w.quantity}`}{/if}</TableHeadCell
              >
              <TableBodyCell title={w.attack?.label} class="px-2 py-2"
                >{shown(w.attack, true)}{#if onroll && w.damage?.attackRoll}{@const attackRoll =
                    w.damage.attackRoll}
                  <RollButton
                    label="attaque, {w.weapon.name}"
                    onclick={() => onroll(attackRoll)}
                  />{/if}</TableBodyCell
              >
              <TableBodyCell class="px-2 py-2">
                {damage(w.damage)}{#if w.damage}<Marks
                    notes={valueMarks(w.damage.id)}
                  />{/if}{#if onroll && w.damage?.roll}{@const dmRoll = w.damage.roll}
                  <RollButton
                    label="DM, {w.weapon.name}"
                    onclick={() => onroll(dmRoll)}
                  />{/if}{#each w.damage?.situational ?? [] as s, k (k)}<span
                    class="block text-xs text-gray-500 dark:text-gray-400"
                    >{situationalText(s)}</span
                  >{/each}
              </TableBodyCell>
              <TableBodyCell class="px-2 py-2">
                {range(
                  w.range,
                )}{#if w.bonus?.total !== undefined}{` · ${signed(w.bonus.total)}`}{/if}
              </TableBodyCell>
            </TableBodyRow>
          {/each}
        </TableBody>
      </Table>
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
      />
      <p class="mt-2 text-sm">
        XP restante : {model.experience ?? "—"}
        {#if play && characterId !== undefined}
          · <a
            class="underline"
            href={`${characterPath(characterId)}#progression`}
            onclick={follow}
          >
            Fin de séance : inscrire l’XP gagnée
          </a>
        {/if}
      </p>
    </Block>

    <Block title="Traits distinctifs" notes={marksAt(notes, ...steps("traits"))}>
      {#each [{ title: "Avantages", group: model.advantages }, { title: "Désavantages", group: model.disadvantages }] as { title, group } (title)}
        <Heading tag="h4" class="text-base text-gray-500 dark:text-gray-400">
          {title}
          {signed(group.total)}
        </Heading>
        {#if group.traits.length > 0}
          <ul class="space-y-1">
            {#each group.traits as t (t.id)}
              <li>{t.name} ({signed(t.points)})</li>
            {/each}
          </ul>
        {:else}
          <p class="text-sm text-gray-500 dark:text-gray-400">Aucun.</p>
        {/if}
      {/each}
      {#if model.sequels.length > 0}
        <Heading tag="h4" class="text-base text-gray-500 dark:text-gray-400"
          >Séquelles subies</Heading
        >
        <ul class="space-y-1">
          {#each model.sequels as sequel (sequel.id)}
            <li>{sequel.name}</li>
          {/each}
        </ul>
      {/if}
    </Block>
  </div>

  <div class="grid items-start gap-4 md:col-span-2 md:grid-cols-2 lg:col-span-12">
    <EquipmentSection
      notes={marksAt(notes, ...steps("equipment"))}
      items={equipment.items ?? []}
      weapons={catalog.weapons.filter((w) => w.attackMode.id !== "unarmed")}
      findings={sheet.inPlay}
      kept={play !== null && play.equipment !== null}
      onitems={play ? (items) => equip({ items }) : undefined}
      onforget={() => play?.forgetEquipment()}
    >
      {#snippet worn(editing: boolean)}
        {#if editing}
          <fieldset class="flex flex-col gap-2">
            <legend class="mb-2 text-sm font-medium text-gray-900 dark:text-white"
              >Protections</legend
            >
            {#each catalog.protections as protection (protection.id)}
              <Checkbox
                checked={equipment.protections?.includes(protection.id) ?? false}
                onchange={(e) =>
                  wear(
                    protection.id,
                    e.currentTarget instanceof HTMLInputElement && e.currentTarget.checked,
                  )}
              >
                {protection.name}
              </Checkbox>
            {/each}
          </fieldset>
        {:else}
          <li>
            Protections :
            {model.protections.join(", ") || "aucune"}
          </li>
        {/if}
      {/snippet}
    </EquipmentSection>

    <NoteList {notes} />
  </div>
</div>
