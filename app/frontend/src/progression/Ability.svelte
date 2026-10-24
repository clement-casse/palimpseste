<script lang="ts">
  // A capacité, named by its identifier (ADR ENG-VALUE-IDS). Today it shows
  // its voie and rank; its name, description and effects will show here.
  import { type Abilities, abilityLabel } from "./progression";

  let { catalog, id }: { catalog: Abilities; id: string } = $props();
</script>

<span data-ability={id}>{abilityLabel(catalog, id)}</span>
