<script lang="ts">
  import { type User, getUser } from "./api";
  import CharacterPage from "./character/CharacterPage.svelte";
  import Characters from "./Characters.svelte";
  import { characterId, route } from "./router.svelte";
  import SignIn from "./SignIn.svelte";

  const USER_KEY = "palimpseste.user";

  function rememberedUser(): number | null {
    try {
      const id = Number(localStorage.getItem(USER_KEY));
      return Number.isInteger(id) && id > 0 ? id : null;
    } catch {
      return null;
    }
  }

  let userId = $state(rememberedUser());
  let user = $state<User | null>(null);

  function signIn(id: number | null) {
    try {
      if (id === null) localStorage.removeItem(USER_KEY);
      else localStorage.setItem(USER_KEY, String(id));
    } catch {
      // Storage unavailable: the choice lasts for this page only.
    }
    userId = id;
  }

  $effect(() => {
    user = null;
    if (userId !== null) getUser(userId).then((found) => (user = found), () => signIn(null));
  });

  const editing = $derived(characterId(route.path));
</script>

<main class={{ wide: editing !== null }}>
  <h1>Palimpseste</h1>
  {#if userId === null}
    <SignIn onSignIn={signIn} />
  {:else if user}
    {#if editing !== null}
      <CharacterPage id={editing} author={user.name} />
    {:else}
      <Characters {user} onSignOut={() => signIn(null)} />
    {/if}
  {/if}
</main>
