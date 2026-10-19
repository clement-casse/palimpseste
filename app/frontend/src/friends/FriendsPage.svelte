<script lang="ts">
  // The user's friends (ADR FE-FRIENDS), each leading to their characters,
  // and the invitation links that make new ones.
  import { A, Heading, Listgroup, ListgroupItem, P, Spinner } from "flowbite-svelte";

  import { listFriends } from "../api";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { follow, friendPath } from "../router.svelte";
  import InviteCard from "./InviteCard.svelte";

  const friends = listFriends();
</script>

<div class="flex flex-col gap-6">
  <section class="flex flex-col gap-4">
    <Heading tag="h2" class="text-2xl">Amis</Heading>
    <P size="sm" class="text-gray-500 dark:text-gray-400">
      Vos amis voient les fiches de vos personnages, et vous les leurs, sans pouvoir les modifier.
    </P>
    {#await friends}
      <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
    {:then friends}
      {#if friends.length === 0}
        <P class="text-gray-500 dark:text-gray-400">
          Aucun ami pour l’instant : envoyez un lien d’invitation.
        </P>
      {:else}
        <Listgroup class="flex-col">
          {#each friends as friend (friend.id)}
            <ListgroupItem class="flex-col items-start gap-1 py-3">
              <A href={friendPath(friend.id)} onclick={follow} class="font-semibold">{friend.nickname}</A>
              {#if friend.profile}
                <span class="w-full truncate text-sm text-gray-500 dark:text-gray-400">
                  {friend.profile.split("\n")[0]}
                </span>
              {/if}
            </ListgroupItem>
          {/each}
        </Listgroup>
      {/if}
    {:catch error}
      <ErrorMessage {error} />
    {/await}
  </section>

  <InviteCard />
</div>
