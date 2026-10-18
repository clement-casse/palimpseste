<script lang="ts">
  // Puts every resource of the sheet back to its maximum, once confirmed.
  import type { PlayState } from "./play.svelte";

  let { play }: { play: PlayState } = $props();
</script>

<button
  type="button"
  class="restore"
  onclick={() => {
    if (confirm("Remettre toutes les ressources au maximum ?")) play.reset();
  }}
>
  Tout restaurer
</button>

<style>
  .restore {
    min-height: 2.75rem;
    margin-left: auto;
  }
</style>
