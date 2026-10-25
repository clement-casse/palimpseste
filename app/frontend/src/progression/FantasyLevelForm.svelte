<script lang="ts">
  // One step of a COF2 level-up being prepared (ADR FE-PROGRESSION): what
  // the engine offers within the rules first, anything else under "Hors
  // règles", which the engine then reports (ADR FE-OVERRIDES).
  import {
    Accordion,
    AccordionItem,
    Button,
    Helper,
    Listgroup,
    ListgroupItem,
    P,
    Select,
  } from "flowbite-svelte";

  import { valueOf } from "../character/edits";
  import type {
    FantasyCatalog,
    FantasyDetails,
    FantasyDocument,
    LevelUpEntry,
  } from "../wasm/palimpseste_wasm";
  import Ability from "./Ability.svelte";
  import AbilityOptionPicker from "./AbilityOptionPicker.svelte";
  import { askingSpeciality } from "./progression";
  import SpecialityField from "./SpecialityField.svelte";

  let {
    choices = $bindable(),
    catalog,
    details,
    number,
    step,
  }: {
    choices: FantasyDocument;
    catalog: FantasyCatalog;
    details: FantasyDetails;
    /** The level the level-up reaches. */
    number: number;
    step: string;
  } = $props();

  const index = $derived(number - 2);
  const levelUp = $derived<LevelUpEntry>(choices.levelUps?.[index] ?? {});
  const offer = $derived(details.levels.find((line) => line.level === number));
  const acquired = $derived(levelUp.abilities ?? []);
  const forgotten = $derived(levelUp.forgotten ?? []);
  const held = $derived(new Set(details.abilitiesHeld.map((h) => h.ability)));
  const exchanges = $derived(new Map(catalog.orphanExchanges.map((e) => [e.id, e.name])));

  /** Changes the level-up prepared, through the document. */
  function change(update: (entry: LevelUpEntry) => LevelUpEntry) {
    const levelUps = [...(choices.levelUps ?? [])];
    levelUps[index] = update({ ...levelUps[index] });
    choices.levelUps = levelUps;
  }

  const acquire = (id: string) =>
    change((entry) => ({ ...entry, abilities: [...(entry.abilities ?? []), id] }));
  const unacquire = (id: string) =>
    change((entry) => ({ ...entry, abilities: (entry.abilities ?? []).filter((a) => a !== id) }));
  const forget = (id: string) =>
    change((entry) => ({ ...entry, forgotten: [...(entry.forgotten ?? []), id] }));
  const unforget = (id: string) =>
    change((entry) => ({ ...entry, forgotten: (entry.forgotten ?? []).filter((a) => a !== id) }));
  const exchange = (id: string | undefined) => change((entry) => ({ ...entry, orphan: id }));

  const REASONS: Record<string, string> = {
    free: "gratuite, elle ne s’oublie pas",
    "level-one": "rang 1 du profil pris au niveau 1, elle ne s’oublie pas",
  };

  /** Picks a capacité from a "Hors règles" select, then clears it. */
  function picked(event: Event, then: (id: string) => void) {
    const id = valueOf(event);
    if (event.currentTarget instanceof HTMLSelectElement) event.currentTarget.value = "";
    if (id) then(id);
  }
</script>

{#if step === "abilities"}
  <P weight="semibold">
    Points de capacité : {offer?.pointsSpent ?? 0} / {offer?.pointsAvailable ?? 2}
  </P>
  {#if acquired.length > 0}
    <Listgroup class="flex-col" aria-label="Capacités acquises">
      {#each acquired as id (id)}
        <ListgroupItem class="justify-between gap-2 text-gray-900 dark:text-white">
          <Ability {catalog} {id} />
          <Button size="xs" color="alternative" onclick={() => unacquire(id)}>Retirer</Button>
        </ListgroupItem>
      {/each}
    </Listgroup>
  {/if}
  {#if offer && offer.acquirable.length > 0}
    <Listgroup class="flex-col" aria-label="Capacités à acquérir">
      {#each offer.acquirable as entry (entry.ability)}
        <ListgroupItem class="justify-between gap-2 text-gray-900 dark:text-white">
          <Ability {catalog} id={entry.ability} />
          <Button size="xs" onclick={() => acquire(entry.ability)}>
            Acquérir ({entry.cost} pt{entry.cost > 1 ? "s" : ""})
          </Button>
        </ListgroupItem>
      {/each}
    </Listgroup>
  {:else}
    <Helper class="text-sm">Plus rien à acquérir dans les règles à ce niveau.</Helper>
  {/if}
  <Accordion flush>
    <AccordionItem headingTag="h3" classes={{ content: "space-y-3 py-3" }}>
      {#snippet header()}Hors règles : autres capacités{/snippet}
      <Select placeholder="" aria-label="Acquérir hors règles" onchange={(e) => picked(e, acquire)}>
        <option value="">Acquérir une autre capacité…</option>
        {#each catalog.abilities.filter((a) => !held.has(a.id) && !acquired.includes(a.id)) as ability (ability.id)}
          <option value={ability.id}>{ability.label}</option>
        {/each}
      </Select>
    </AccordionItem>
  </Accordion>
{:else if step === "options"}
  {@const lines = details.abilityOptions.filter((line) => line.progression === number)}
  {@const asking = askingSpeciality(catalog, details.abilitiesHeld, number)}
  {#if lines.length + asking.length > 0}
    {#each lines as line (line.ability)}
      <AbilityOptionPicker bind:choices {catalog} {line} />
    {/each}
    {#each asking as ability (ability)}
      <SpecialityField bind:choices {catalog} {ability} />
    {/each}
  {:else}
    <Helper class="text-sm">Aucune capacité de ce niveau ne demande d’option.</Helper>
  {/if}
{:else if step === "orientation"}
  <Helper class="text-sm">
    Facultatif : oubliez {offer?.forgetMax ?? 1} capacité{(offer?.forgetMax ?? 1) > 1 ? "s" : ""} au
    plus ; ses points reviennent au niveau.
  </Helper>
  {#if forgotten.length > 0}
    <Listgroup class="flex-col" aria-label="Capacités oubliées">
      {#each forgotten as id (id)}
        <ListgroupItem class="justify-between gap-2 text-gray-900 dark:text-white">
          <Ability {catalog} {id} />
          <Button size="xs" color="alternative" onclick={() => unforget(id)}>La garder</Button>
        </ListgroupItem>
      {/each}
    </Listgroup>
  {/if}
  {#if offer && offer.forgettable.length > 0}
    <Listgroup class="flex-col" aria-label="Capacités à oublier">
      {#each offer.forgettable.filter((id) => !forgotten.includes(id)) as id (id)}
        <ListgroupItem class="justify-between gap-2 text-gray-900 dark:text-white">
          <Ability {catalog} {id} />
          <Button size="xs" color="alternative" onclick={() => forget(id)}>Oublier</Button>
        </ListgroupItem>
      {/each}
    </Listgroup>
  {/if}
  {#if offer && offer.protected.length > 0}
    <Helper class="text-sm">
      {#each offer.protected as entry (entry.ability)}
        <span class="block"><Ability {catalog} id={entry.ability} /> : {REASONS[entry.reason] ?? entry.reason}.</span>
      {/each}
    </Helper>
  {/if}
  <Accordion flush>
    <AccordionItem headingTag="h3" classes={{ content: "space-y-3 py-3" }}>
      {#snippet header()}Hors règles : oublier une autre capacité{/snippet}
      <Select placeholder="" aria-label="Oublier hors règles" onchange={(e) => picked(e, forget)}>
        <option value="">Oublier une autre capacité…</option>
        {#each details.abilitiesHeld.filter((h) => !forgotten.includes(h.ability)) as h (h.ability)}
          <option value={h.ability}>{catalog.abilities.find((a) => a.id === h.ability)?.label ?? h.ability}</option>
        {/each}
      </Select>
    </AccordionItem>
  </Accordion>
{:else if step === "orphan"}
  {#if levelUp.orphan}
    <P>
      Point orphelin échangé contre {exchanges.get(levelUp.orphan) ?? levelUp.orphan}.
      <Button size="xs" color="alternative" onclick={() => exchange(undefined)}>Annuler l’échange</Button>
    </P>
  {:else if offer?.orphanAllowed}
    <Helper class="text-sm">Plus aucune capacité de rang 1 ou 2 à acquérir : échangez le point restant.</Helper>
    <div class="flex flex-wrap gap-2">
      {#each catalog.orphanExchanges as option (option.id)}
        <Button size="sm" onclick={() => exchange(option.id)}>{option.name}</Button>
      {/each}
    </div>
  {:else}
    <Helper class="text-sm">
      Le point orphelin s’échange une fois toutes les capacités de rang 1 et 2 acquises.
    </Helper>
    <Accordion flush>
      <AccordionItem headingTag="h3" classes={{ content: "space-y-3 py-3" }}>
        {#snippet header()}Hors règles : échanger un point{/snippet}
        <Select placeholder="" aria-label="Échanger hors règles" onchange={(e) => picked(e, exchange)}>
          <option value="">Échanger un point contre…</option>
          {#each catalog.orphanExchanges as option (option.id)}
            <option value={option.id}>{option.name}</option>
          {/each}
        </Select>
      </AccordionItem>
    </Accordion>
  {/if}
{/if}
