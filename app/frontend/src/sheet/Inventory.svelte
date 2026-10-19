<script lang="ts">
  // The items carried in play, counted up and down as they are used (torches,
  // javelots…). An item used up stays, struck out, until it is removed.
  // Without `onchange`, only read.
  import { Button } from "flowbite-svelte";
  import { MinusOutline, PlusOutline } from "flowbite-svelte-icons";

  import type { ItemEntry } from "../wasm/palimpseste_wasm";

  let {
    items,
    names,
    onchange,
  }: {
    items: ItemEntry[];
    /** Weapon names by identifier. */
    names: Map<string, string>;
    onchange?: (items: ItemEntry[]) => void;
  } = $props();

  function label(item: ItemEntry): string {
    return item.kind === "weapon" ? (names.get(item.id) ?? item.id) : item.name;
  }

  function count(index: number, quantity: number) {
    onchange?.(items.map((item, i) => (i === index ? { ...item, quantity } : item)));
  }
</script>

{#each items as item, i (i)}
  <li class="flex items-center gap-1.5">
    <span class={["flex-1", item.quantity === 0 && "text-gray-500 line-through dark:text-gray-400"]}>
      {label(item)}
    </span>
    <span
      class={["text-gray-500 tabular-nums dark:text-gray-400", item.quantity === 0 && "line-through"]}
    >
      ×{item.quantity}
    </span>
    {#if onchange}
      <Button
        color="alternative"
        class="h-10 w-10 p-0 print:hidden"
        aria-label="Retirer 1 {label(item)}"
        disabled={item.quantity === 0}
        onclick={() => count(i, item.quantity - 1)}
      >
        <MinusOutline size="sm" />
      </Button>
      <Button
        color="alternative"
        class="h-10 w-10 p-0 print:hidden"
        aria-label="Ajouter 1 {label(item)}"
        onclick={() => count(i, item.quantity + 1)}
      >
        <PlusOutline size="sm" />
      </Button>
    {/if}
  </li>
{/each}
