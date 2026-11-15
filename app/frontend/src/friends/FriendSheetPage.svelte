<script lang="ts">
  // A friend's character, on its sheet read only (ADR FE-SHEET).
  import { A } from "flowbite-svelte";

  import Loader from "../lib/components/Loader.svelte";
  import { getFriend, getFriendCharacter } from "../api";
  import ErrorMessage from "../ErrorMessage.svelte";
  import { FRIENDS, follow } from "../router.svelte";
  import CharacterSheet from "../sheet/CharacterSheet.svelte";

  let { friend, id }: { friend: number; id: number } = $props();

  const load = (friend: number, id: number) =>
    Promise.all([getFriend(friend), getFriendCharacter(friend, id)]);
</script>

<!-- Another character is another sheet, loaded anew. -->
{#key `${friend}/${id}`}
  <Loader load={load(friend, id)}>
    {#snippet children([owner, character])}
      <CharacterSheet {character} friend={owner} />
    {/snippet}
    {#snippet failed(error)}
      <ErrorMessage {error} />
      <A href={FRIENDS} onclick={follow}>← Amis</A>
    {/snippet}
  </Loader>
{/key}
