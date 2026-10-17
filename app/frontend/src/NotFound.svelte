<script lang="ts">
  import { follow } from "./router.svelte";
</script>

<section>
  <h2>Page introuvable</h2>
  <p>Cette adresse ne correspond à aucune page.</p>
  <a href="/" onclick={follow}>← Personnages</a>
</section>
