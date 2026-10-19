<script lang="ts">
  // A friend (ADR FE-FRIENDS): their profile, their characters to read, and
  // ending the friendship, for both of them.
  import {
    A,
    Badge,
    Button,
    Card,
    Heading,
    Listgroup,
    ListgroupItem,
    Modal,
    P,
    Spinner,
  } from "flowbite-svelte";
  import { ArrowLeftOutline } from "flowbite-svelte-icons";

  import { type Friend, getFriend, listFriendCharacters, removeFriend } from "../api";
  import { ENGINES } from "../engines";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { FRIENDS, follow, friendSheetPath, navigate } from "../router.svelte";

  let { id }: { id: number } = $props();

  let confirming = $state(false);
  let removing = $state(false);
  let failure = $state<unknown>(null);

  const load = (id: number) => Promise.all([getFriend(id), listFriendCharacters(id)]);

  function remove(friend: Friend) {
    removing = true;
    failure = null;
    removeFriend(friend.id).then(
      () => navigate(FRIENDS, { replace: true }),
      (error: unknown) => {
        failure = error;
        removing = false;
        confirming = false;
      },
    );
  }
</script>

<section class="flex flex-col gap-4">
  <A href={FRIENDS} onclick={follow} class="gap-1 self-start"><ArrowLeftOutline size="sm" /> Amis</A>
  {#key id}
    {#await load(id)}
      <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
    {:then [friend, characters]}
      <Heading tag="h2" class="text-2xl">{friend.nickname}</Heading>
      {#if friend.profile}<P class="whitespace-pre-line">{friend.profile}</P>{/if}

      <Card size="xl" class="gap-4 p-6">
        <Heading tag="h3" class="text-lg">Personnages</Heading>
        {#if characters.length === 0}
          <P class="text-gray-500 dark:text-gray-400">Aucun personnage pour l’instant.</P>
        {:else}
          <Listgroup class="flex-col">
            {#each characters as character (character.id)}
              <ListgroupItem class="flex-wrap gap-x-3 gap-y-2 py-3">
                <A href={friendSheetPath(friend.id, character.id)} onclick={follow} class="font-semibold">
                  {character.name}
                </A>
                <Badge color="gray">{ENGINES[character.engine]}</Badge>
              </ListgroupItem>
            {/each}
          </Listgroup>
        {/if}
      </Card>

      {#if failure}<ErrorMessage error={failure} />{/if}
      <Button color="red" outline class="self-start" onclick={() => (confirming = true)}>
        Retirer des amis
      </Button>
      <Modal bind:open={confirming} title="Retirer {friend.nickname} de vos amis ?" size="sm">
        <P>
          Vous ne verrez plus ses personnages, et {friend.nickname} ne verra plus les vôtres. Un nouveau
          lien d’invitation vous rendra amis à nouveau.
        </P>
        <div class="flex flex-wrap gap-2">
          <Button color="red" disabled={removing} onclick={() => remove(friend)}>Retirer</Button>
          <Button color="alternative" onclick={() => (confirming = false)}>Annuler</Button>
        </div>
      </Modal>
    {:catch error}
      <ErrorMessage {error} />
    {/await}
  {/key}
</section>
