<script lang="ts">
  // A roll resolved, as the engine returned it (ADR FE-ROLLS): the dice,
  // every line of the total, the total, and what may still apply. Whether
  // it succeeds is the Game Master's to say.
  import { Badge } from "flowbite-svelte";

  import { signed } from "$lib/character/edits";
  import type { RollOutcome } from "$lib/engine";
  import { optionText, rangeFrom } from "$lib/format";

  let { outcome }: { outcome: RollOutcome } = $props();

  const range = $derived(rangeFrom(outcome.criticalFrom ?? 20));
</script>

<div class="flex flex-col gap-3">
  <p class="flex flex-wrap items-center gap-2" aria-label="Dés lancés">
    {#each outcome.dice as die, i (i)}
      <Badge
        large
        color={die.kept ? "primary" : "gray"}
        class={die.kept ? "font-bold" : "line-through"}
      >
        d{die.faces} : {die.face}{#if !die.kept}<span class="sr-only"> (écarté)</span>{/if}
      </Badge>
    {/each}
    {#if outcome.mode === "bonus"}<span class="text-sm text-gray-500 dark:text-gray-400"
        >dé bonus : le meilleur est gardé</span
      >{/if}
    {#if outcome.mode === "malus"}<span class="text-sm text-gray-500 dark:text-gray-400"
        >dé malus : le moins bon est gardé</span
      >{/if}
  </p>
  {#if outcome.natural || outcome.critical}
    <p class="flex flex-wrap gap-2">
      {#if outcome.natural === "twenty"}<Badge color="green">20 naturel</Badge>{/if}
      {#if outcome.natural === "one"}<Badge color="red">1 naturel</Badge>{/if}
      {#if outcome.critical}
        <Badge color="yellow">
          {outcome.kind === "damage" ? "DM critiques" : `Dans la zone de critique (${range})`}
        </Badge>
      {/if}
    </p>
  {/if}
  <ul class="divide-y divide-gray-200 text-sm dark:divide-gray-700" aria-label="Calcul">
    {#each outcome.lines as line, i (i)}
      <li class="flex items-baseline justify-between gap-3 py-1">
        <span>
          {line.kind === "dice" ? `Dés (${line.label})` : line.label}
          {#if line.kind !== "dice" && line.dice.length > 0}
            <span class="text-gray-500 dark:text-gray-400"
              >({line.dice.map((d) => `d${d.faces} : ${d.face}`).join(", ")})</span
            >
          {/if}
          {#if line.origin}<span class="block text-xs text-gray-500 dark:text-gray-400"
              >{line.origin}</span
            >{/if}
        </span>
        <span class="font-semibold tabular-nums">{signed(line.amount)}</span>
      </li>
    {/each}
  </ul>
  <p
    class="flex items-baseline justify-between gap-3 border-t-2 border-primary-700 pt-2 dark:border-primary-400"
  >
    <span class="font-bold">Total</span>
    <output class="text-3xl font-bold tabular-nums" aria-label="Total">{outcome.total}</output>
  </p>
  {#if outcome.setAside.length > 0}
    <div class="text-sm">
      <p class="font-semibold">Non cumulés : un seul bonus de compétence par source</p>
      <ul class="list-inside list-disc text-gray-500 line-through dark:text-gray-400">
        {#each outcome.setAside as option, i (i)}
          <li>{optionText(option)} — {option.origin}</li>
        {/each}
      </ul>
    </div>
  {/if}
  {#if outcome.unpicked.length > 0}
    <div class="text-sm">
      <p class="font-semibold">Peut aussi s’appliquer, à annoncer au MJ</p>
      <ul class="list-inside list-disc">
        {#each outcome.unpicked as option, i (i)}
          <li>
            {optionText(option)}
            <span class="text-gray-500 dark:text-gray-400">— {option.origin}</span>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</div>
