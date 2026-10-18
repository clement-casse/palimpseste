<script lang="ts">
  // A COC2 character at the table: the échelle de santé counted in the bar,
  // then what the player reads during a session, with the equipment in
  // play. Every number comes from the engine.
  import { namesOf, signed } from "../character/edits";
  import type {
    ContemporainCatalog,
    ContemporainDetails,
    ContemporainDocument,
    ContemporainEquipment,
    Sheet,
    ValueLine,
  } from "../wasm/palimpseste_wasm";
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

<p class="identity">
  {identity}<Marks notes={marksAt(notes, ...steps("domaine-and-age"))} />
</p>

<div class="play-bar">
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

<div class="play-sections">
  <section class="block">
    <h3>Caractéristiques<Marks notes={marksAt(notes, ...steps("characteristics"))} /></h3>
    <Characteristics characteristics={sheet.characteristics} {notes} />
  </section>

  <section class="block">
    <h3>Combat<Marks notes={marksAt(notes, ...steps("secondary"))} /></h3>
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
    <table class="weapons">
      <thead>
        <tr><th>Arme</th><th>Att.</th><th>DM</th><th>Critique</th></tr>
      </thead>
      <tbody>
        {#each weapons as w (w.weapon.id)}
          <tr>
            <th scope="row">{w.weapon.name}{#if w.quantity > 1}{` ×${w.quantity}`}{/if}</th>
            <td title={w.attack?.label}>{shown(w.attack, true)}</td>
            <td>
              {damage(w.damage)}{#if w.damage}<Marks notes={valueMarks(w.damage.id)} />{/if}
            </td>
            <td>
              {range(w.range)}{#if w.bonus?.total !== undefined}{` · ${signed(w.bonus.total)}`}{/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </section>

  <section class="block">
    <h3>Voies<Marks notes={marksAt(notes, ...steps("voies"))} /></h3>
    <VoieRanks voies={choices.voies ?? []} names={namesOf(catalog.voies)} {notes} />
  </section>

  <section class="block">
    <h3>Traits distinctifs<Marks notes={marksAt(notes, ...steps("traits"))} /></h3>
    {#each [true, false] as advantage (advantage)}
      {@const listed = traits.filter((t) => t.advantage === advantage)}
      <h4>
        {advantage ? "Avantages" : "Désavantages"}
        {signed(advantage ? details.advantages : details.disadvantages)}
      </h4>
      {#if listed.length > 0}
        <ul class="listing">
          {#each listed as t (t.id)}
            <li>{t.name} ({signed(t.points)})</li>
          {/each}
        </ul>
      {:else}
        <p class="hint">Aucun.</p>
      {/if}
    {/each}
  </section>

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
        <fieldset class="field">
          <legend>Protections</legend>
          {#each catalog.protections as protection (protection.id)}
            <label>
              <input
                type="checkbox"
                checked={equipment.protections?.includes(protection.id) ?? false}
                onchange={(e) =>
                  wear(protection.id, e.currentTarget instanceof HTMLInputElement && e.currentTarget.checked)}
              />
              {protection.name}
            </label>
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
