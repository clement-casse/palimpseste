<script lang="ts">
  // Puts every resource of the sheet back to its maximum, once confirmed.
  import { Button } from "flowbite-svelte";

  import type { PlayState } from "./play.svelte";

  let { play }: { play: PlayState } = $props();
</script>

<Button
  color="alternative"
  class="ms-auto min-h-11 print:hidden"
  onclick={() => {
    if (confirm("Remettre toutes les ressources au maximum ?")) play.reset();
  }}
>
  Tout restaurer
</Button>
