<script lang="ts">
  // The items carried in play, counted up and down as they are used (torches,
  // javelots…). An item used up stays, struck out, until it is removed.
  import type { ItemEntry } from "../wasm/palimpseste_wasm";

  let {
    items,
    names,
    onchange,
  }: {
    items: ItemEntry[];
    /** Weapon names by identifier. */
    names: Map<string, string>;
    onchange: (items: ItemEntry[]) => void;
  } = $props();

  function label(item: ItemEntry): string {
    return item.kind === "weapon" ? (names.get(item.id) ?? item.id) : item.name;
  }

  function count(index: number, quantity: number) {
    onchange(items.map((item, i) => (i === index ? { ...item, quantity } : item)));
  }
</script>

{#each items as item, i (i)}
  <li class={["item", item.quantity === 0 && "spent"]}>
    <span class="name">{label(item)}</span>
    <span class="quantity">×{item.quantity}</span>
    <button
      type="button"
      aria-label="Retirer 1 {label(item)}"
      disabled={item.quantity === 0}
      onclick={() => count(i, item.quantity - 1)}>−</button
    >
    <button type="button" aria-label="Ajouter 1 {label(item)}" onclick={() => count(i, item.quantity + 1)}
      >+</button
    >
  </li>
{/each}

<style>
  .item {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }

  .name {
    flex: 1;
  }

  .quantity {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .spent .name,
  .spent .quantity {
    color: var(--muted);
    text-decoration: line-through;
  }

  button {
    width: 2.5rem;
    height: 2.5rem;
    padding: 0;
    font-size: 1.2rem;
    line-height: 1;
  }
</style>
