<script lang="ts">
  import { Alert, Button, Card, Heading, P } from "flowbite-svelte";

  import Loader from "$lib/components/Loader.svelte";
  import { listProviders, signInUrl } from "$lib/api";
  import { takeQuery } from "$lib/router.svelte";

  // A failed sign-in comes back as `/login?signin=failed`: say so once.
  const failed = takeQuery("signin") === "failed";
  const providers = listProviders();
</script>

<Card size="md" class="mx-auto gap-4 p-6">
  <Heading tag="h2" class="text-2xl">Connexion</Heading>
  {#if failed}<Alert color="red">La connexion a échoué.</Alert>{/if}
  <P size="sm" class="text-gray-500 dark:text-gray-400">
    Première connexion ? Vous créerez votre compte à l’étape suivante.
  </P>
  <Loader load={providers}>
    {#snippet children(providers)}
      {#if providers.length === 0}
        <P>Aucun fournisseur d’identité n’est configuré.</P>
      {/if}
      <ul class="flex flex-col gap-2">
        {#each providers as provider (provider.id)}
          <!-- Signing in leaves the app for the provider: a link, not a script. -->
          <li><Button class="w-full" href={signInUrl(provider.id)}>Se connecter avec {provider.name}</Button></li>
        {/each}
      </ul>
    {/snippet}
  </Loader>
</Card>
