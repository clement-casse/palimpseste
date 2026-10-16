<script lang="ts">
  // The choices of a COF2 character, one section per creation step. The
  // options the rules allow come first; anything else stays reachable under
  // "Hors règles", and the engine reports it.
  import type {
    Characteristic,
    FantasyCatalog,
    FantasyDetails,
    FantasyDocument,
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
    document: FantasyDocument;
    catalog: FantasyCatalog;
    sheet: Sheet;
    step: string;
    edit: (change: (document: FantasyDocument) => void) => void;
  } = $props();

  const voieNames = $derived(namesOf(catalog.voies));
  const gearNames = $derived(
    namesOf([...catalog.weapons, ...catalog.armours, ...catalog.shields]),
  );
  const profil = $derived(catalog.profils.find((p) => p.id === document.profil));
  const peuple = $derived(catalog.peuples.find((p) => p.id === document.peuple));
  const series = $derived(catalog.series.find((s) => s.id === document.series));
  const details = $derived(sheet.details as FantasyDetails);

  // Characteristics switched to a value outside the series.
  let custom = $state<Partial<Record<Characteristic, boolean>>>({});

  function assigned(c: Characteristic): number | undefined {
    return document.seriesValues?.[c] ?? undefined;
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
    edit((d) => {
      d.seriesValues ??= {};
      d.seriesValues[c] = choice === "" ? undefined : Number(choice);
    });
  }

  function assignOther(c: Characteristic, value: number | undefined) {
    edit((d) => {
      d.seriesValues ??= {};
      d.seriesValues[c] = value;
    });
  }

  function keyMark(c: Characteristic): string {
    const key = profil?.keyCharacteristics.some((slot) => slot.includes(c));
    const magic = profil?.magicCharacteristic === c;
    return [key ? "clé" : "", magic ? "magie" : ""].filter(Boolean).join(", ");
  }

  const otherVoies = $derived(
    (document.voies ?? [])
      .map((voie) => voie.id)
      .filter((id) => !profil?.voies.includes(id) && !details.peupleSlotOptions.includes(id)),
  );

  function setVoie(id: string, ranks: number[]) {
    edit((d) => setRanks(d, id, ranks));
  }

  /** Whether an armour or shield is within the profil's limit; without a profil, every one is. */
  function within(list: { id: string }[], id: string, limit: string | undefined): boolean {
    if (!profil) return true;
    if (limit === undefined) return false;
    return list.findIndex((x) => x.id === id) <= list.findIndex((x) => x.id === limit);
  }

  function addGear(id: string) {
    edit((d) => {
      if (catalog.armours.some((a) => a.id === id)) d.armour = id;
      else if (catalog.shields.some((s) => s.id === id)) d.shield = id;
      else {
        const items = (d.items ??= []);
        const item = items.find((i) => i.kind === "weapon" && i.id === id);
        if (item) item.quantity += 1;
        else items.push({ kind: "weapon", id, quantity: 1 });
      }
    });
  }

  function setItems(items: ItemEntry[]) {
    edit((d) => {
      d.items = items;
    });
  }
</script>

{#if step === "profil"}
  <label class="field">
    Profil
    <select
      value={document.profil ?? ""}
      onchange={(e) => {
        const id = valueOf(e);
        edit((d) => (d.profil = id || undefined));
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
    </select>
  </label>
  {#if profil}
    <dl class="facts">
      <dt>Famille</dt>
      <dd>{details.family?.name}</dd>
      <dt>Voies</dt>
      <dd>{profil.voies.map((id) => voieNames.get(id)).join(", ")}</dd>
      <dt>Caractéristiques clés</dt>
      <dd>{profil.keyCharacteristics.map((slot) => slot.join(" ou ")).join(", ")}</dd>
      <dt>Caractéristique de magie</dt>
      <dd>{profil.magicCharacteristic ?? "aucune"}</dd>
      <dt>Armure</dt>
      <dd>{profil.armourLimit ? `jusqu'à ${gearNames.get(profil.armourLimit)}` : "aucune"}</dd>
      <dt>Bouclier</dt>
      <dd>{profil.shieldLimit ? `jusqu'au ${gearNames.get(profil.shieldLimit)}` : "aucun"}</dd>
      <dt>Armes maîtrisées</dt>
      <dd>{profil.masteredWeapons.map((id) => gearNames.get(id)).join(", ")}</dd>
    </dl>
  {/if}
{:else if step === "peuple"}
  <label class="field">
    Peuple
    <select
      value={document.peuple ?? ""}
      onchange={(e) => {
        const id = valueOf(e);
        edit((d) => (d.peuple = id || undefined));
      }}
    >
      <option value="">— Choisir —</option>
      {#each catalog.peuples as p (p.id)}
        <option value={p.id}>{p.name}</option>
      {/each}
    </select>
  </label>
  {#if peuple}
    <dl class="facts">
      <dt>Bonus (+1)</dt>
      <dd>
        {peuple.bonusOptions.length === 7
          ? "une des deux caractéristiques les plus basses"
          : peuple.bonusOptions.join(" ou ")}
      </dd>
      <dt>Malus (-1)</dt>
      <dd>{peuple.malusOptions.length ? peuple.malusOptions.join(" ou ") : "aucun"}</dd>
      <dt>Voie de peuple</dt>
      <dd>
        {peuple.voie
          ? voieNames.get(peuple.voie)
          : "celle de l'humain ou d'un elfe, selon l'éducation"}
      </dd>
    </dl>
  {/if}
{:else if step === "characteristics"}
  <fieldset class="field">
    <legend>Série de valeurs</legend>
    {#each catalog.series as s (s.id)}
      <label>
        <input
          type="radio"
          name="series"
          checked={document.series === s.id}
          onchange={() => edit((d) => (d.series = s.id))}
        />
        {s.name} ({s.values.map(signed).join(", ")})
      </label>
    {/each}
  </fieldset>
  <table class="assign">
    <tbody>
      {#each CHARACTERISTICS as c (c)}
        <tr>
          <th scope="row">{c} <small>{keyMark(c)}</small></th>
          <td>
            <select
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
            </select>
            {#if seriesChoice(c) === "other"}
              <input
                type="number"
                step="1"
                aria-label="Valeur hors série de {c}"
                value={assigned(c) ?? ""}
                onchange={(e) => assignOther(c, integerOf(e))}
              />
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if peuple}
    <label class="field">
      Bonus de peuple (+1)
      <select
        value={document.peupleBonus ?? ""}
        onchange={(e) => {
          const c = valueOf(e) as Characteristic | "";
          edit((d) => (d.peupleBonus = c || undefined));
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
      </select>
    </label>
    <label class="field">
      Malus de peuple (-1)
      <select
        value={document.peupleMalus ?? ""}
        onchange={(e) => {
          const c = valueOf(e) as Characteristic | "";
          edit((d) => (d.peupleMalus = c || undefined));
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
      </select>
    </label>
  {/if}
{:else if step === "voies"}
  <p class="hint">
    Deux voies du profil au rang 1, et la voie de peuple au rang 1. Un mage prend en plus un rang 2.
  </p>
  <h3>Voies du profil</h3>
  {#if profil}
    <ul class="voies">
      {#each profil.voies as id (id)}
        <li>
          <span>{voieNames.get(id)}</span>
          <RankPicker ranks={ranksOf(document.voies, id)} onchange={(r) => setVoie(id, r)} />
        </li>
      {/each}
    </ul>
  {:else}
    <p class="hint">Choisissez d'abord un profil.</p>
  {/if}
  <h3>Voie de peuple</h3>
  {#if details.peupleSlotOptions.length > 0}
    <ul class="voies">
      {#each details.peupleSlotOptions as id (id)}
        <li>
          <span>{voieNames.get(id)}</span>
          <RankPicker ranks={ranksOf(document.voies, id)} onchange={(r) => setVoie(id, r)} />
        </li>
      {/each}
    </ul>
  {:else}
    <p class="hint">Choisissez d'abord un peuple.</p>
  {/if}
  <details class="out-of-rules" open={otherVoies.length > 0}>
    <summary>Hors règles : autres voies</summary>
    <ul class="voies">
      {#each otherVoies as id (id)}
        <li>
          <span>{voieNames.get(id) ?? id}</span>
          <RankPicker ranks={ranksOf(document.voies, id)} onchange={(r) => setVoie(id, r)} />
        </li>
      {/each}
    </ul>
    <select
      aria-label="Ajouter une voie"
      onchange={(e) => {
        const id = valueOf(e);
        if (e.currentTarget instanceof HTMLSelectElement) e.currentTarget.value = "";
        if (id) setVoie(id, [1]);
      }}
    >
      <option value="">Ajouter une voie…</option>
      {#each catalog.voies.filter((v) => !ranksOf(document.voies, v.id).length) as v (v.id)}
        <option value={v.id}>{v.name}</option>
      {/each}
    </select>
  </details>
{:else if step === "equipment"}
  <div class="row">
    <label class="field">
      Armure
      <select
        value={document.armour ?? ""}
        onchange={(e) => {
          const id = valueOf(e);
          edit((d) => (d.armour = id || undefined));
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
      </select>
    </label>
    <label class="field">
      Bouclier
      <select
        value={document.shield ?? ""}
        onchange={(e) => {
          const id = valueOf(e);
          edit((d) => (d.shield = id || undefined));
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
      </select>
    </label>
  </div>
  {#if profil}
    <h3>Équipement de départ du profil</h3>
    <ul class="starting">
      {#each profil.startingEquipment as line, i (i)}
        <li>
          {#each line as bundle, j (j)}
            {#if j > 0}<em>ou</em>{/if}
            <span class="bundle">
              {#each bundle as item, k (k)}
                <span class="item">
                  {#if item.quantity > 1}{item.quantity} ×{/if}
                  {#each item.options as id, m (id)}
                    {#if m > 0}/{/if}
                    <button type="button" class="link" onclick={() => addGear(id)}>
                      {gearNames.get(id)}
                    </button>
                  {/each}
                </span>
              {/each}
            </span>
          {/each}
        </li>
      {/each}
    </ul>
    <p class="hint">
      Plus le sac d'aventurier. Un objet spécial se négocie avec le MJ, contre un objet de valeur
      proche.
    </p>
  {/if}
  <h3>Objets portés</h3>
  <Items items={document.items ?? []} weapons={catalog.weapons} onchange={setItems} />
  <label class="field">
    Bourse ({catalog.purseDice} pa, à lancer)
    <input
      type="number"
      step="1"
      value={document.purse === undefined ? "" : document.purse / 10}
      onchange={(e) => {
        const pa = integerOf(e);
        edit((d) => (d.purse = pa === undefined ? undefined : pa * 10));
      }}
    />
  </label>
  {#if details.money}<p class="hint">Argent : {details.money}</p>{/if}
{/if}
