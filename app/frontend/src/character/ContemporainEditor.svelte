<script lang="ts">
  // The choices of a COC2 character, one section per creation step. The book
  // allows deferring and revisiting every choice; budgets and limits come
  // from the engine, which reports what goes beyond them.
  import type {
    Characteristic,
    ContemporainCatalog,
    ContemporainDetails,
    ContemporainDocument,
    ItemEntry,
    Sheet,
  } from "../wasm/palimpseste_wasm";
  import { CHARACTERISTICS, integerOf, namesOf, ranksOf, setRanks, signed, valueOf } from "./edits";
  import Items from "./Items.svelte";
  import RankPicker from "./RankPicker.svelte";

  let {
    document,
    catalog,
    sheet,
    step,
    edit,
  }: {
    document: ContemporainDocument;
    catalog: ContemporainCatalog;
    sheet: Sheet;
    step: string;
    edit: (change: (document: ContemporainDocument) => void) => void;
  } = $props();

  const voieNames = $derived(namesOf(catalog.voies));
  const domaine = $derived(catalog.domaines.find((d) => d.id === document.domaine));
  const details = $derived(sheet.details as ContemporainDetails);
  const domaineVoies = $derived(new Set(domaine?.slots.flat() ?? []));

  function total(id: string): string {
    const value = sheet.values.find((line) => line.id === id)?.total;
    return value === undefined ? "?" : String(value);
  }

  function pointsOf(id: string): number | undefined {
    return document.traits?.find((t) => t.id === id)?.points;
  }

  function setTrait(id: string, points: number | undefined) {
    edit((d) => {
      const traits = (d.traits ??= []);
      const index = traits.findIndex((t) => t.id === id);
      if (points === undefined) {
        if (index >= 0) traits.splice(index, 1);
      } else if (index >= 0) {
        traits[index].points = points;
      } else {
        traits.push({ id, points });
      }
    });
  }

  function setPoints(c: Characteristic, points: number | undefined) {
    edit((d) => {
      d.caracPoints ??= {};
      d.caracPoints[c] = points ?? 0;
    });
  }

  function setItems(items: ItemEntry[]) {
    edit((d) => {
      d.items = items;
    });
  }

  function toggleProtection(id: string, worn: boolean) {
    edit((d) => {
      const protections = (d.protections ??= []);
      d.protections = worn ? [...protections, id] : protections.filter((p) => p !== id);
    });
  }

  const startingWeapons = $derived(
    catalog.voies.filter(
      (voie) => voie.startingWeapon && ranksOf(document.voies, voie.id).length > 0,
    ),
  );
</script>

{#if step === "domaine-and-age"}
  <div class="row">
    <label class="field">
      Âge
      <input
        type="number"
        min="0"
        step="1"
        value={document.age ?? ""}
        onchange={(e) => {
          const age = integerOf(e);
          edit((d) => (d.age = age !== undefined && age >= 0 ? age : undefined));
        }}
      />
    </label>
    <p class="hint">Tranche d'âge : {details.ageBracket ?? "—"}</p>
  </div>
  <label class="field">
    Domaine
    <select
      value={document.domaine ?? ""}
      onchange={(e) => {
        const id = valueOf(e);
        edit((d) => (d.domaine = id || undefined));
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
    </select>
  </label>
  {#if domaine}
    <p class="hint">
      Retenez au moins {domaine.minimumRetained} des voies du domaine :
      {domaine.slots.map((slot) => slot.map((id) => voieNames.get(id)).join(" ou ")).join(", ")}.
    </p>
  {/if}
{:else if step === "voies"}
  <p class="budget">
    Points de capacité : {details.capacityPointsSpent} / {total("coc.capacity-points")} · Voies :
    {details.voieCount} / {total("coc.max-voies")} · Rang maximum : {total("coc.max-rank")}
  </p>
  <ul class="voies">
    {#each [...catalog.voies].sort((a, b) => Number(domaineVoies.has(b.id)) - Number(domaineVoies.has(a.id))) as voie (voie.id)}
      <li>
        <span>
          {voie.name}
          {#if domaineVoies.has(voie.id)}<small class="tag">domaine</small>{/if}
        </span>
        <RankPicker
          ranks={ranksOf(document.voies, voie.id)}
          onchange={(ranks) => edit((d) => setRanks(d, voie.id, ranks))}
        />
      </li>
    {/each}
  </ul>
{:else if step === "traits"}
  <p class="budget">
    Avantages {signed(details.advantages)} · Désavantages {signed(details.disadvantages)} · Solde
    {signed(details.advantages + details.disadvantages)} · Classe sociale :
    {details.socialClass ?? "contradictoire"}
  </p>
  {#each [true, false] as advantage (advantage)}
    <h3>{advantage ? "Avantages" : "Désavantages"}</h3>
    <ul class="traits">
      {#each catalog.traits.filter((t) => t.advantage === advantage) as t (t.id)}
        {@const points = pointsOf(t.id)}
        <li>
          <label>
            <input
              type="checkbox"
              checked={points !== undefined}
              onchange={(e) =>
                setTrait(
                  t.id,
                  e.currentTarget instanceof HTMLInputElement && e.currentTarget.checked
                    ? t.allowedPoints[0]
                    : undefined,
                )}
            />
            {t.name}
          </label>
          {#if points !== undefined}
            <select
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
            </select>
          {/if}
        </li>
      {/each}
    </ul>
  {/each}
{:else if step === "characteristics"}
  <p class="budget">
    Points de caractéristique : {details.caracPointsSpent} / {total("coc.carac-points")}
  </p>
  <table class="assign">
    <tbody>
      {#each CHARACTERISTICS as c (c)}
        <tr>
          <th scope="row">{c}</th>
          <td>
            <input
              type="number"
              step="1"
              aria-label="Points attribués à {c}"
              value={document.caracPoints?.[c] ?? 0}
              onchange={(e) => setPoints(c, integerOf(e))}
            />
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
{:else if step === "equipment"}
  {#if startingWeapons.length > 0}
    <h3>Équipement de départ des voies</h3>
    <ul class="starting">
      {#each startingWeapons as voie (voie.id)}
        <li>{voie.name} : une arme ({voie.startingWeapon?.name})</li>
      {/each}
    </ul>
  {/if}
  <h3>Protections</h3>
  <ul class="protections">
    {#each catalog.protections as protection (protection.id)}
      <li>
        <label>
          <input
            type="checkbox"
            checked={document.protections?.includes(protection.id) ?? false}
            onchange={(e) =>
              toggleProtection(
                protection.id,
                e.currentTarget instanceof HTMLInputElement && e.currentTarget.checked,
              )}
          />
          {protection.name}
        </label>
      </li>
    {/each}
  </ul>
  <p class="hint">Aucune voie ne donne de protection : en porter une se négocie avec le MJ.</p>
  <h3>Objets portés</h3>
  <Items
    items={document.items ?? []}
    weapons={catalog.weapons.filter((w) => w.attackMode.id !== "unarmed")}
    onchange={setItems}
  />
{/if}
