<script lang="ts">
  // One step of a COC2 spending of XP being prepared (ADR FE-PROGRESSION):
  // what the engine offers within the rules first, anything else under "Hors
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
    ContemporainCatalog,
    ContemporainDetails,
    ContemporainDocument,
    TraitChangeEntry,
    XpSpendingEntry,
  } from "../wasm/palimpseste_wasm";
  import Ability from "./Ability.svelte";
  import { askingSpeciality } from "./progression";
  import RankFivePicker from "./RankFivePicker.svelte";
  import SpecialityField from "./SpecialityField.svelte";

  let {
    choices = $bindable(),
    catalog,
    details,
    number,
    step,
  }: {
    choices: ContemporainDocument;
    catalog: ContemporainCatalog;
    details: ContemporainDetails;
    /** The spending's number, from 1. */
    number: number;
    step: string;
  } = $props();

  const index = $derived(number - 1);
  const spending = $derived<XpSpendingEntry>(choices.xpSpendings?.[index] ?? {});
  const offer = $derived(details.spendings.find((line) => line.number === number));
  const bought = $derived(spending.abilities ?? []);
  const changes = $derived(spending.traits ?? []);
  const held = $derived(new Set(details.abilitiesHeld.map((h) => h.ability)));
  const traitNames = $derived(new Map(catalog.traits.map((t) => [t.id, t.name])));

  /** Changes the spending prepared, through the document. */
  function change(update: (entry: XpSpendingEntry) => XpSpendingEntry) {
    const spendings = [...(choices.xpSpendings ?? [])];
    spendings[index] = update({ ...spendings[index] });
    choices.xpSpendings = spendings;
  }

  const buy = (id: string) =>
    change((entry) => ({ ...entry, abilities: [...(entry.abilities ?? []), id] }));
  const unbuy = (id: string) =>
    change((entry) => ({ ...entry, abilities: (entry.abilities ?? []).filter((a) => a !== id) }));
  function setTrait(trait: TraitChangeEntry) {
    change((entry) => ({
      ...entry,
      traits: [...(entry.traits ?? []).filter((t) => t.id !== trait.id), trait],
    }));
  }
  const unsetTrait = (id: string) =>
    change((entry) => ({ ...entry, traits: (entry.traits ?? []).filter((t) => t.id !== id) }));

  function describe(trait: TraitChangeEntry): string {
    const name = traitNames.get(trait.id) ?? trait.id;
    return trait.points === undefined || trait.points === null
      ? `${name} abandonné`
      : `${name} à ${trait.points}`;
  }

  /** Picks an entry from a "Hors règles" select, then clears it. */
  function picked(event: Event, then: (id: string) => void) {
    const id = valueOf(event);
    if (event.currentTarget instanceof HTMLSelectElement) event.currentTarget.value = "";
    if (id) then(id);
  }

  let otherTrait = $state("");
</script>

{#if step === "ranks"}
  <P weight="semibold">XP restants : {offer?.xpLeft ?? 0}</P>
  {#if bought.length > 0}
    <Listgroup class="flex-col" aria-label="Rangs achetés">
      {#each bought as id (id)}
        <ListgroupItem class="justify-between gap-2 text-gray-900 dark:text-white">
          <Ability {catalog} {id} />
          <Button size="xs" color="alternative" onclick={() => unbuy(id)}>Retirer</Button>
        </ListgroupItem>
      {/each}
    </Listgroup>
  {/if}
  {@const asking = askingSpeciality(catalog, details.abilitiesHeld, number)}
  {#if asking.length > 0}
    {#each asking as ability (ability)}
      <SpecialityField bind:choices {catalog} {ability} />
    {/each}
  {/if}
  {#if offer && offer.acquirable.length > 0}
    <Listgroup class="flex-col" aria-label="Rangs à acheter">
      {#each offer.acquirable as entry (entry.ability)}
        <ListgroupItem class="justify-between gap-2 text-gray-900 dark:text-white">
          <Ability {catalog} id={entry.ability} />
          <Button size="xs" onclick={() => buy(entry.ability)}>Acheter ({entry.cost} XP)</Button>
        </ListgroupItem>
      {/each}
    </Listgroup>
  {:else}
    <Helper class="text-sm">Plus aucun rang à acheter dans les règles avec l’XP restante.</Helper>
  {/if}
  <Helper class="text-sm">Chaque rang doit être accordé par le MJ et convenir au personnage.</Helper>
  <Accordion flush>
    <AccordionItem headingTag="h3" classes={{ content: "space-y-3 py-3" }}>
      {#snippet header()}Hors règles : autres rangs{/snippet}
      <Select placeholder="" aria-label="Acheter hors règles" onchange={(e) => picked(e, buy)}>
        <option value="">Acheter un autre rang…</option>
        {#each catalog.abilities.filter((a) => !held.has(a.id) && !bought.includes(a.id)) as ability (ability.id)}
          <option value={ability.id}>{ability.label}</option>
        {/each}
      </Select>
    </AccordionItem>
  </Accordion>
{:else if step === "traits"}
  {#if changes.length > 0}
    <Listgroup class="flex-col" aria-label="Traits changés">
      {#each changes as trait (trait.id)}
        <ListgroupItem class="justify-between gap-2 text-gray-900 dark:text-white">
          <span>{describe(trait)}</span>
          <Button size="xs" color="alternative" onclick={() => unsetTrait(trait.id)}>Retirer</Button>
        </ListgroupItem>
      {/each}
    </Listgroup>
  {/if}
  <div class="flex flex-wrap gap-2">
    {#if offer?.classRise}
      {@const rise = offer.classRise}
      <Button size="sm" onclick={() => setTrait({ id: rise.id, points: rise.points })}>
        Monter d’une classe sociale ({rise.cost} XP)
      </Button>
    {/if}
    {#if offer?.cure !== undefined && offer?.cure !== null}
      <Button size="sm" onclick={() => setTrait({ id: "coc.trait.toxicomane" })}>
        Guérir la toxicomanie ({offer.cure} XP)
      </Button>
    {/if}
  </div>
  {#if !offer?.classRise && (offer?.cure === undefined || offer?.cure === null)}
    <Helper class="text-sm">Ni classe sociale à gagner ni toxicomanie à guérir.</Helper>
  {/if}
  <Accordion flush>
    <AccordionItem headingTag="h3" classes={{ content: "space-y-3 py-3" }}>
      {#snippet header()}Hors règles : autre changement de trait{/snippet}
      <Select placeholder="" aria-label="Trait à changer" bind:value={otherTrait}>
        <option value="">Choisir un trait…</option>
        {#each catalog.traits as trait (trait.id)}
          <option value={trait.id}>{trait.name}</option>
        {/each}
      </Select>
      {#if otherTrait}
        {@const trait = catalog.traits.find((t) => t.id === otherTrait)}
        <div class="flex flex-wrap gap-2">
          {#each trait?.allowedPoints ?? [] as points (points)}
            <Button size="xs" color="alternative" onclick={() => setTrait({ id: otherTrait, points })}>
              À {points}
            </Button>
          {/each}
          <Button size="xs" color="alternative" onclick={() => setTrait({ id: otherTrait })}>Abandonner</Button>
        </div>
      {/if}
    </AccordionItem>
  </Accordion>
{:else if step === "characteristics"}
  {@const lines = details.rankFive.filter((line) => line.progression === number && line.choice.length > 0)}
  {#if lines.length > 0}
    {#each lines as line (line.voie)}
      <RankFivePicker bind:choices {catalog} {line} />
    {/each}
  {:else}
    <Helper class="text-sm">Aucun rang 5 acheté ici ne demande de choix.</Helper>
  {/if}
{/if}
