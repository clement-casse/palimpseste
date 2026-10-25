<script lang="ts">
  // A capacité, named by its identifier (ADR ENG-VALUE-IDS): its name with
  // its voie and rank, or, `short` under its voie, its rank and name.
  import { type Abilities, abilityLabel } from "./progression";

  let { catalog, id, short = false }: { catalog: Abilities; id: string; short?: boolean } =
    $props();

  const entry = $derived(catalog.abilities.find((a) => a.id === id));
</script>

<span data-ability={id}>{short && entry ? `${entry.rank}. ${entry.name}` : abilityLabel(catalog, id)}</span>
