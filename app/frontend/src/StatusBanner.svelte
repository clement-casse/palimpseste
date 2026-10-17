<script lang="ts">
  // What the user should know about the app itself: a new version waiting,
  // or no network.
  import { pwa, useNewVersion } from "./pwa.svelte";
</script>

{#if pwa.waiting}
  <p role="status">
    Une nouvelle version est disponible.
    <button type="button" onclick={useNewVersion}>Recharger</button>
  </p>
{/if}
{#if !pwa.online}
  <p role="status" class="offline">Hors ligne : les modifications ne sont pas enregistrées.</p>
{/if}

<style>
  p {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1rem;
    align-items: center;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--accent);
    border-radius: 0.25rem;
  }

  .offline {
    border-color: var(--error);
    color: var(--error);
  }
</style>
