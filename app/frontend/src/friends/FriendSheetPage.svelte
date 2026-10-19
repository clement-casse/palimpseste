<script lang="ts">
  // A friend's character, on its sheet read only (ADR FE-SHEET).
  import { A, Spinner } from "flowbite-svelte";

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
  {#await load(friend, id)}
    <p class="flex items-center gap-2"><Spinner size="4" /> Chargement…</p>
  {:then [owner, character]}
    <CharacterSheet {character} friend={owner} />
  {:catch error}
    <ErrorMessage {error} />
    <A href={FRIENDS} onclick={follow}>← Amis</A>
  {/await}
{/key}
