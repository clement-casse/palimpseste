<script lang="ts">
  // What the user should know about the app itself: a new version waiting,
  // or no network.
  import { Alert, Button } from "flowbite-svelte";

  import { pwa, useNewVersion } from "./pwa.svelte";
</script>

{#if pwa.waiting}
  <Alert role="status" class="my-2 flex flex-wrap items-center gap-x-4 gap-y-2">
    Une nouvelle version est disponible.
    <Button size="xs" onclick={useNewVersion}>Recharger</Button>
  </Alert>
{/if}
{#if !pwa.online}
  <Alert role="status" color="red" class="my-2">
    Hors ligne : les modifications ne sont pas enregistrées.
  </Alert>
{/if}
