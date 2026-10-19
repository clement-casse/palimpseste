<script lang="ts">
  // Items carried: weapons of the engine's tables, and free text items the
  // rules never check (D-26).
  import { Button, Input, Listgroup, ListgroupItem, Select } from "flowbite-svelte";

  import type { ItemEntry } from "../wasm/palimpseste_wasm";
  import { U32_MAX, integerOf, valueOf } from "./edits";

  let {
    items,
    weapons,
    onchange,
  }: {
    items: ItemEntry[];
    weapons: { id: string; name: string }[];
    onchange: (items: ItemEntry[]) => void;
  } = $props();

  let other = $state("");

  const names = $derived(new Map(weapons.map((weapon) => [weapon.id, weapon.name])));

  function label(item: ItemEntry): string {
    return item.kind === "weapon" ? (names.get(item.id) ?? item.id) : item.name;
  }

  function setQuantity(index: number, quantity: number) {
    onchange(items.map((item, i) => (i === index ? { ...item, quantity } : item)));
  }

  function typeQuantity(event: Event, index: number) {
    const quantity = integerOf(event, items[index].quantity, { min: 0, max: U32_MAX, blank: false });
    if (typeof quantity === "number") setQuantity(index, quantity);
  }

  function addWeapon(event: Event) {
    const id = valueOf(event);
    if (event.currentTarget instanceof HTMLSelectElement) event.currentTarget.value = "";
    if (!id) return;
    const index = items.findIndex((item) => item.kind === "weapon" && item.id === id);
    if (index >= 0) setQuantity(index, items[index].quantity + 1);
    else onchange([...items, { kind: "weapon", id, quantity: 1 }]);
  }

  function addOther(event: SubmitEvent) {
    event.preventDefault();
    const name = other.trim();
    if (!name) return;
    onchange([...items, { kind: "other", name, quantity: 1 }]);
    other = "";
  }
</script>

<div class="space-y-3">
  {#if items.length > 0}
    <Listgroup class="flex-col">
      {#each items as item, i (i)}
        <ListgroupItem class="flex-wrap gap-2 text-gray-900 dark:text-white">
          <span class="flex-1">{label(item)}</span>
          <div class="w-20">
            <Input
              type="number"
              min="0"
              size="sm"
              aria-label="Quantité"
              value={item.quantity}
              onchange={(event) => typeQuantity(event, i)}
            />
          </div>
          <Button size="xs" color="alternative" onclick={() => onchange(items.filter((_, j) => j !== i))}>
            Retirer
          </Button>
        </ListgroupItem>
      {/each}
    </Listgroup>
  {/if}
  <div class="flex flex-wrap gap-2">
    <Select placeholder="" class="w-auto" aria-label="Ajouter une arme" onchange={addWeapon}>
      <option value="">Ajouter une arme…</option>
      {#each weapons as weapon (weapon.id)}
        <option value={weapon.id}>{weapon.name}</option>
      {/each}
    </Select>
    <form class="flex flex-1 gap-2" onsubmit={addOther}>
      <Input class="min-w-40" aria-label="Autre objet" placeholder="Autre objet" bind:value={other} />
      <Button type="submit" disabled={!other.trim()}>Ajouter</Button>
    </form>
  </div>
</div>
