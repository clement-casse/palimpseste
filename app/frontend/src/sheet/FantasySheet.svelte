<script lang="ts">
  // A COF2 character at the table: PV, PM, PC and DR counted in the bar,
  // then what the player reads during a session, with the equipment in
  // play. Every number comes from the engine.
  import { namesOf, signed, valueOf } from "../character/edits";
  import type {
    FantasyCatalog,
    FantasyDetails,
    FantasyDocument,
    FantasyEquipment,
    Sheet,
    ValueLine,
  } from "../wasm/palimpseste_wasm";
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
    play: PlayState;
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
    play.equip({ engine: "fantasy", ...equipment, ...change });
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

<p class="identity">
  {identity}<Marks notes={marksAt(notes, ...steps("profil", "peuple"))} />
</p>

<div class="play-bar">
  {#each resources as resource (resource.id)}
    <Counter
      label={resource.short}
      title={resource.label}
      max={resource.total}
      used={play.used(resource.id, resource.total)}
      die={resource.dice}
      notes={valueMarks(resource.id)}
      onchange={(used) => play.set(resource.id, used, resource.total)}
    />
  {/each}
  <Restore {play} />
</div>

<div class="play-sections">
  <section class="block">
    <h3>Caractéristiques<Marks notes={marksAt(notes, ...steps("characteristics"))} /></h3>
    <Characteristics characteristics={sheet.characteristics} {notes} />
  </section>

  <section class="block">
    <h3>Combat<Marks notes={marksAt(notes, ...steps("derived"))} /></h3>
    <ul class="tiles">
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
      <table class="weapons">
        <thead>
          <tr><th>Arme</th><th>Usage</th><th>DM</th><th>Critique</th></tr>
        </thead>
        <tbody>
          {#each weapons as { weapon, quantity, damages, critical } (weapon.id)}
            {#each damages as dm, i (dm.id)}
              <tr>
                {#if i === 0}
                  <th scope="row" rowspan={damages.length}>
                    {weapon.name}{#if quantity > 1}{` ×${quantity}`}{/if}
                  </th>
                {/if}
                <td>{weapon.uses[i]?.name ?? ""}</td>
                <td>{damage(dm)}<Marks notes={valueMarks(dm.id)} /></td>
                {#if i === 0}
                  <td rowspan={damages.length}>
                    {range(critical)}{#if critical}<Marks notes={valueMarks(critical.id)} />{/if}
                  </td>
                {/if}
              </tr>
            {/each}
          {/each}
        </tbody>
      </table>
    {/if}
  </section>

  <section class="block">
    <h3>Voies<Marks notes={marksAt(notes, ...steps("voies"))} /></h3>
    <VoieRanks voies={choices.voies ?? []} names={namesOf(catalog.voies)} {notes} />
  </section>

  <EquipmentSection
    notes={marksAt(notes, ...steps("equipment"))}
    items={equipment.items ?? []}
    weapons={catalog.weapons}
    findings={sheet.inPlay}
    kept={play.equipment !== null}
    onitems={(items) => equip({ items })}
    onforget={() => play.forgetEquipment()}
  >
    {#snippet worn(editing: boolean)}
      {#if editing}
        <label class="field">
          Armure
          <select
            value={equipment.armour ?? ""}
            onchange={(e) => equip({ armour: valueOf(e) || undefined })}
          >
            <option value="">Aucune</option>
            {#each catalog.armours as a (a.id)}
              <option value={a.id}>{a.name} (DEF {signed(a.defence)})</option>
            {/each}
          </select>
        </label>
        <label class="field">
          Bouclier
          <select
            value={equipment.shield ?? ""}
            onchange={(e) => equip({ shield: valueOf(e) || undefined })}
          >
            <option value="">Aucun</option>
            {#each catalog.shields as s (s.id)}
              <option value={s.id}>{s.name} (DEF {signed(s.defence)})</option>
            {/each}
          </select>
        </label>
      {:else}
        <li>Armure : {armour ? `${armour.name} (DEF ${signed(armour.defence)})` : "aucune"}</li>
        <li>Bouclier : {shield ? `${shield.name} (DEF ${signed(shield.defence)})` : "aucun"}</li>
        <li>Bourse : {details.money ?? "—"}<Marks notes={valueMarks("cof.money")} /></li>
      {/if}
    {/snippet}
  </EquipmentSection>

  <NoteList {notes} />
</div>
