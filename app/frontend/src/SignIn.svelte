<script lang="ts">
  import { listProviders, signInUrl } from "./api";
  import ErrorMessage from "./ErrorMessage.svelte";
  import { takeQuery } from "./router.svelte";

  // A failed sign-in comes back as `/login?signin=failed`: say so once.
  const failed = takeQuery("signin") === "failed";
  const providers = listProviders();
</script>

<section>
  <h2>Connexion</h2>
  {#if failed}<p class="error" role="alert">La connexion a échoué.</p>{/if}
  <p class="hint">Première connexion ? Vous créerez votre compte à l’étape suivante.</p>
  {#await providers}
    <p>Chargement…</p>
  {:then providers}
    {#if providers.length === 0}
      <p>Aucun fournisseur d’identité n’est configuré.</p>
    {/if}
    <ul class="choices">
      {#each providers as provider (provider.id)}
        <!-- Signing in leaves the app for the provider: a link, not a script. -->
        <li><a class="button" href={signInUrl(provider.id)}>Se connecter avec {provider.name}</a></li>
      {/each}
    </ul>
  {:catch error}
    <ErrorMessage {error} />
  {/await}
</section>
