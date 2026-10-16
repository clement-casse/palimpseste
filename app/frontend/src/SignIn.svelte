<script lang="ts">
  import { type Provider, listProviders, signInUrl } from "./api";

  // A failed sign-in comes back as `/?signin=failed`: say so once.
  const failed = new URLSearchParams(location.search).get("signin") === "failed";
  if (failed) history.replaceState(null, "", location.pathname);

  let providers = $state<Provider[] | null>(null);
  let error = $state<string | null>(failed ? "La connexion a échoué." : null);

  listProviders().then(
    (all) => (providers = all),
    (e: Error) => (error = e.message),
  );
</script>

<section>
  <h2>Connexion</h2>
  {#if error}<p class="error">{error}</p>{/if}
  {#if providers?.length === 0}<p>Aucun fournisseur d’identité n’est configuré.</p>{/if}
  <ul class="choices">
    {#each providers ?? [] as provider (provider.id)}
      <li>
        <button type="button" onclick={() => location.assign(signInUrl(provider.id))}>
          Se connecter avec {provider.name}
        </button>
      </li>
    {/each}
  </ul>
</section>
