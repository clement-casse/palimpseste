<script lang="ts">
  // One roll, from its die to its total (ADR FE-ROLLS). The state and what
  // is done with it are `RollController`'s: this component only shows them.
  import {
    Accordion,
    AccordionItem,
    Alert,
    Button,
    ButtonToggle,
    ButtonToggleGroup,
    Checkbox,
    Heading,
    Helper,
    Input,
    Label,
    Modal,
  } from "flowbite-svelte";
  import { MinusOutline, PlusOutline } from "flowbite-svelte-icons";
  import { untrack } from "svelte";

  import { I32_MAX, I32_MIN, integerOf, signed } from "$lib/character/edits";
  import type { Document, Mode, Play, RollOutcome } from "$lib/engine";
  import { type Placed, type PlacedDie, RollController } from "./rollController.svelte";
  import RollResult from "./RollResult.svelte";
  import { optionText } from "$lib/format";

  let {
    choices,
    play,
    roll,
    critical = false,
    onrolled,
    onchain,
    onclose,
  }: {
    choices: Document;
    /** The equipment in play and the damage taken, as the device knows them. */
    play: Play;
    /** The identifier of the roll, from a line of the sheet. */
    roll: string;
    /** Whether the DM start as those of a critical success. */
    critical?: boolean;
    onrolled: (outcome: RollOutcome) => void;
    /** Opens the DM roll following an attack. */
    onchain: (roll: string, critical: boolean) => void;
    onclose: () => void;
  } = $props();

  const MODES: { mode: Mode; label: string }[] = [
    { mode: "plain", label: "Dé normal" },
    { mode: "bonus", label: "Dé bonus" },
    { mode: "malus", label: "Dé malus" },
  ];

  // The dialog is one roll: the page recreates it for another.
  const rolling = untrack(() => new RollController(choices, play, roll, critical, onrolled));
  const offer = rolling.offer;

  // The group only reads its value when drawn: it is redrawn when the mode
  // changes, and when a second tap on the mode chosen would leave none.
  let redraws = $state(0);
  function choose(selected: string | string[] | null) {
    const mode = MODES.find((option) => option.mode === selected);
    if (mode) rolling.chooseMode(mode.mode);
    else redraws += 1;
  }

  function launch(event: SubmitEvent) {
    event.preventDefault();
    rolling.launch();
  }
</script>

{#snippet options(placed: Placed[])}
  {#each placed as { option, index } (index)}
    <Checkbox bind:checked={rolling.picked[index]}>
      <span>
        {optionText(option)}
        <span class="block text-xs text-gray-500 dark:text-gray-400">{option.origin}</span>
      </span>
    </Checkbox>
  {/each}
{/snippet}

{#snippet dice(placed: PlacedDie[])}
  {#each placed as { die, index } (index)}
    <Checkbox
      bind:checked={() => rolling.dicePicked[index], (picked) => rolling.pickDie(index, picked)}
    >
      <span>
        Dé bonus : {die.scope}
        <span class="block text-xs text-gray-500 dark:text-gray-400">{die.origin}</span>
      </span>
    </Checkbox>
  {/each}
{/snippet}

<Modal open title={offer?.name ?? "Jet"} size="md" oncancel={onclose}>
  {#if !offer}
    <Alert color="red"
      >Ce jet ne peut pas être préparé : {rolling.offered.ok ? "" : rolling.offered.error}</Alert
    >
  {:else}
    <form class="flex flex-col gap-4" onsubmit={launch}>
      {#if offer.kind !== "damage"}
        <section class="flex flex-col gap-2" aria-label="Dé">
          {#key `${rolling.mode} ${redraws}`}
            <ButtonToggleGroup name="Dé" value={rolling.mode} onSelect={choose}>
              {#each MODES as option (option.mode)}
                <ButtonToggle value={option.mode}>{option.label}</ButtonToggle>
              {/each}
            </ButtonToggleGroup>
          {/key}
          {#if rolling.grantedDice.length > 0}
            <ul class="text-sm" aria-label="Dés bonus acquis">
              {#each rolling.grantedDice as die, i (i)}
                <li>
                  Dé bonus : {die.scope}
                  <span class="text-gray-500 dark:text-gray-400">— {die.origin}</span>
                </li>
              {/each}
            </ul>
          {/if}
        </section>
      {/if}

      <div class="flex items-center justify-between gap-4">
        <Heading tag="h4" class="w-auto text-xl">Base : {signed(offer.base)}</Heading>
        <Button type="submit" class="min-h-11 shrink-0"
          >{rolling.outcome ? "Relancer" : "Lancer"}
          {offer.kind === "damage" ? "les DM" : "le dé"}</Button
        >
      </div>

      {#if rolling.featured.length + rolling.featuredDice.length > 0}
        <fieldset class="flex flex-col gap-2">
          <legend class="mb-1 font-semibold text-gray-900 dark:text-white"
            >Selon les circonstances</legend
          >
          <Helper
            >Cochez ce qui s’applique à ce jet ; le reste sera rappelé avec le résultat.</Helper
          >
          {@render options(rolling.featured)}
          {@render dice(rolling.featuredDice)}
        </fieldset>
      {/if}

      {#if offer.notes.length > 0}
        <section class="text-sm" aria-label="À savoir">
          <p class="font-semibold text-gray-900 dark:text-white">À savoir</p>
          <ul class="list-inside list-disc">
            {#each offer.notes as note, i (i)}
              <li>
                {note.scope} <span class="text-gray-500 dark:text-gray-400">— {note.origin}</span>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if offer.critical}
        <Checkbox bind:checked={rolling.critical}>Coup critique : {offer.critical}</Checkbox>
      {/if}

      <Accordion flush>
        <AccordionItem>
          {#snippet header()}
            Autres bonus et modificateurs{rolling.foldedInUse > 0
              ? ` (${rolling.foldedInUse} en cours)`
              : ""}
          {/snippet}
          <div class="flex flex-col gap-4">
            {#if rolling.others.length + rolling.otherDice.length > 0}
              <fieldset class="flex flex-col gap-2">
                <legend class="mb-1 font-semibold text-gray-900 dark:text-white"
                  >Pour d’autres caractéristiques</legend
                >
                {@render options(rolling.others)}
                {@render dice(rolling.otherDice)}
              </fieldset>
            {/if}

            <section aria-label="Base">
              <p class="font-semibold text-gray-900 dark:text-white">
                Base : {signed(offer.base)}
              </p>
              <ul class="text-sm text-gray-500 dark:text-gray-400">
                {#each offer.contributions as contribution, i (i)}
                  <li>{contribution.label} {signed(contribution.amount)}</li>
                {/each}
              </ul>
            </section>

            {#if offer.helperBonus != null}
              <div
                class="flex flex-wrap items-center gap-2"
                role="group"
                aria-label={offer.helpersLabel}
              >
                <span class="font-semibold text-gray-900 dark:text-white">{offer.helpersLabel}</span
                >
                <Button
                  color="alternative"
                  class="h-11 w-11 p-0"
                  aria-label="Un allié de moins"
                  disabled={rolling.helpers <= 0}
                  onclick={() => (rolling.helpers -= 1)}
                >
                  <MinusOutline />
                </Button>
                <output class="min-w-8 text-center text-xl font-bold tabular-nums"
                  >{rolling.helpers}</output
                >
                <Button
                  color="alternative"
                  class="h-11 w-11 p-0"
                  aria-label="Un allié de plus"
                  onclick={() => (rolling.helpers += 1)}
                >
                  <PlusOutline />
                </Button>
                <span class="text-sm text-gray-500 dark:text-gray-400"
                  >{signed(offer.helperBonus)} chacun</span
                >
              </div>
            {/if}

            <Label class="flex flex-col gap-2">
              Modificateur du MJ
              <Input
                type="number"
                step="1"
                value={rolling.modifier}
                onchange={(event) => {
                  const typed = integerOf(event, rolling.modifier, {
                    min: I32_MIN,
                    max: I32_MAX,
                    blank: false,
                  });
                  if (typed != null) rolling.modifier = typed;
                }}
              />
            </Label>
          </div>
        </AccordionItem>
      </Accordion>
    </form>

    {#if rolling.failure}<Alert color="red" class="mt-3">Le jet a échoué : {rolling.failure}</Alert
      >{/if}
    <div class="mt-4" aria-live="polite">
      {#if rolling.outcome}
        {@const outcome = rolling.outcome}
        <RollResult {outcome} />
        {#if outcome.damageRoll}
          {@const next = outcome.damageRoll}
          {@const wasCritical = outcome.critical}
          <Button
            color="alternative"
            class="mt-3 min-h-11"
            onclick={() => onchain(next, wasCritical)}
          >
            Lancer les DM{wasCritical ? " (critique)" : ""}
          </Button>
        {/if}
      {/if}
    </div>
  {/if}
</Modal>
