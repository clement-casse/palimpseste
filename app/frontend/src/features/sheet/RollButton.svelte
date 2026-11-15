<script lang="ts">
  // The die beside a value: opens its roll (ADR FE-ROLLS).
  import { Button } from "flowbite-svelte";
  import { CubeSolid } from "flowbite-svelte-icons";

  let {
    label,
    onclick,
  }: { /** What is rolled, such as `Test de FOR`. */ label: string; onclick: () => void } =
    $props();
</script>

<Button
  color="alternative"
  class="h-8 w-8 shrink-0 p-0 print:hidden"
  aria-label="Lancer : {label}"
  title="Lancer : {label}"
  {onclick}
>
  <CubeSolid size="sm" />
</Button>
