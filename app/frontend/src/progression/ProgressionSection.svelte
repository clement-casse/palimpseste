<script lang="ts">
  // The progression of a character in its editor (ADR FE-PROGRESSION): the
  // level-ups (COF2) or spendings of XP (COC2) validated, each opened in its
  // own form, and the next one once the character is complete. For COC2,
  // the XP earned and the sequels acquired in play, saved with the document
  // like every editor change.
  import {
    Badge,
    Button,
    Heading,
    Helper,
    Input,
    Label,
    Listgroup,
    ListgroupItem,
    Modal,
    P,
    Select,
  } from "flowbite-svelte";

  import { U32_MAX, integerOf, valueOf } from "../character/edits";
  import { unsettled, unsettledAll } from "../engines";
  import { characterProgressionPath, follow } from "../router.svelte";
  import { type Catalog, type Document, type Sheet, evaluate } from "../wasm/palimpseste_wasm";
  import Differences from "./Differences.svelte";
  import { loadDraft } from "./drafts";
  import { differences, nextLabel } from "./progression";

  let {
    choices = $bindable(),
    catalog,
    sheet,
    characterId,
  }: { choices: Document; catalog: Catalog; sheet: Sheet; characterId: number } = $props();

  const open = $derived(unsettledAll(sheet));
  const draftPath = $derived.by(() => {
    const draft = loadDraft(characterId);
    return draft ? characterProgressionPath(characterId, draft.fresh ? "new" : draft.number) : null;
  });
  const total = (id: string) => sheet.values.find((line) => line.id === id)?.total;

  // A sequel to record, shown with what it changes before it is.
  let sequel = $state("");
  const preview = $derived.by(() => {
    if (!sequel || choices.engine !== "contemporain") return [];
    const after = structuredClone($state.snapshot(choices));
    if (after.engine !== "contemporain") return [];
    after.acquiredSequels = [...(after.acquiredSequels ?? []), sequel];
    try {
      return differences(sheet, evaluate(after));
    } catch {
      return [];
    }
  });
  const sequelNames = $derived(
    new Map(catalog.engine === "contemporain" ? catalog.sequels.map((s) => [s.id, s.name]) : []),
  );

  function record() {
    if (choices.engine === "contemporain" && sequel) {
      choices.acquiredSequels = [...(choices.acquiredSequels ?? []), sequel];
    }
    sequel = "";
  }

  function forget(id: string) {
    if (choices.engine === "contemporain") {
      choices.acquiredSequels = (choices.acquiredSequels ?? []).filter((s) => s !== id);
    }
  }
</script>

{#if sheet.details.engine === "fantasy"}
  <P weight="semibold">Niveau {sheet.details.level} · dé évolutif d4° : {sheet.details.evolutiveDie}</P>
{:else if choices.engine === "contemporain"}
  <div class="flex flex-wrap items-end gap-3">
    <Label class="flex flex-col gap-2">
      XP gagnée
      <Input
        type="number"
        min="0"
        class="w-32"
        value={choices.experience ?? 0}
        onchange={(e) => {
          const value = integerOf(e, choices.experience ?? 0, { min: 0, max: U32_MAX, blank: false });
          if (value !== null && value !== undefined && choices.engine === "contemporain") choices.experience = value;
        }}
      />
    </Label>
    <Button
      color="alternative"
      onclick={() => {
        if (choices.engine === "contemporain") choices.experience = (choices.experience ?? 0) + 1;
      }}
    >
      +1 XP (fin de séance)
    </Button>
    <P class="pb-2">XP restante : {total("coc.experience") ?? "—"}</P>
  </div>
{/if}

<Heading tag="h3" class="text-lg">{choices.engine === "fantasy" ? "Niveaux" : "Dépenses d’XP"}</Heading>
{#if sheet.progression.length > 0}
  <Listgroup class="flex-col">
    {#each sheet.progression as entry (entry.number)}
      {@const left = unsettled(entry.steps)}
      <ListgroupItem class="flex-wrap justify-between gap-2 text-gray-900 dark:text-white">
        <span class="flex items-center gap-2">
          {entry.name}
          {#if left > 0}<Badge color="yellow">{left} à régler</Badge>{/if}
        </span>
        <Button size="xs" color="alternative" href={characterProgressionPath(characterId, entry.number)} onclick={follow}>
          Ouvrir
        </Button>
      </ListgroupItem>
    {/each}
  </Listgroup>
{:else}
  <Helper class="text-sm">{choices.engine === "fantasy" ? "Niveau 1 : aucun niveau gagné depuis la création." : "Aucune XP dépensée."}</Helper>
{/if}

{#if draftPath}
  <P size="sm">Un brouillon est en cours sur cet appareil. <a class="underline" href={draftPath} onclick={follow}>Le reprendre</a></P>
{/if}
{#if open === 0}
  <Button class="w-fit" href={characterProgressionPath(characterId, "new")} onclick={follow}>{nextLabel(choices)}</Button>
{:else}
  <Button class="w-fit" disabled>{nextLabel(choices)}</Button>
  <Helper class="text-sm">
    Terminez d’abord le personnage : {open} point{open > 1 ? "s" : ""} à régler, choix manquants ou
    règles enfreintes sans dérogation.
  </Helper>
{/if}

{#if choices.engine === "contemporain" && catalog.engine === "contemporain"}
  <Heading tag="h3" class="text-lg">Séquelles subies</Heading>
  {#if (choices.acquiredSequels ?? []).length > 0}
    <Listgroup class="flex-col">
      {#each choices.acquiredSequels ?? [] as id (id)}
        <ListgroupItem class="justify-between gap-2 text-gray-900 dark:text-white">
          <span>{sequelNames.get(id) ?? id}</span>
          <Button size="xs" color="alternative" onclick={() => forget(id)}>Retirer</Button>
        </ListgroupItem>
      {/each}
    </Listgroup>
  {/if}
  <Select
    placeholder=""
    class="w-auto"
    aria-label="Séquelle subie"
    onchange={(e) => {
      sequel = valueOf(e);
      if (e.currentTarget instanceof HTMLSelectElement) e.currentTarget.value = "";
    }}
  >
    <option value="">Inscrire une séquelle subie…</option>
    {#each catalog.sequels.filter((s) => !(choices.engine === "contemporain" && choices.acquiredSequels?.includes(s.id))) as s (s.id)}
      <option value={s.id}>{s.name}</option>
    {/each}
  </Select>
{/if}

<Modal open={sequel !== ""} title={sequelNames.get(sequel) ?? ""} size="sm" oncancel={() => (sequel = "")}>
  <P>Ce que la séquelle change sur la fiche :</P>
  <Differences differences={preview} />
  <div class="flex justify-end gap-2">
    <Button color="alternative" onclick={() => (sequel = "")}>Annuler</Button>
    <Button onclick={record}>Inscrire la séquelle</Button>
  </div>
</Modal>
