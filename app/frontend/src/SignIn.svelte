<script lang="ts">
  import { type User, createUser, listUsers } from "./api";

  let { onSignIn }: { onSignIn: (id: number) => void } = $props();

  let users = $state<User[]>([]);
  let name = $state("");
  let error = $state<string | null>(null);

  listUsers().then(
    (all) => (users = all),
    (e: Error) => (error = e.message),
  );

  function submit(event: SubmitEvent) {
    event.preventDefault();
    createUser(name.trim()).then(
      (user) => onSignIn(user.id),
      (e: Error) => (error = e.message),
    );
  }
</script>

<section>
  <h2>Qui joue ?</h2>
  {#if error}<p class="error">{error}</p>{/if}
  <ul class="choices">
    {#each users as user (user.id)}
      <li><button type="button" onclick={() => onSignIn(user.id)}>{user.name}</button></li>
    {/each}
  </ul>
  <form onsubmit={submit}>
    <input aria-label="Nom" placeholder="Nom" bind:value={name} />
    <button type="submit" disabled={!name.trim()}>Créer</button>
  </form>
</section>
