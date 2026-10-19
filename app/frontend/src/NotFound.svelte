<script lang="ts">
  import { A, Heading, P } from "flowbite-svelte";

  import { follow } from "./router.svelte";
</script>

<section class="space-y-4">
  <Heading tag="h2" class="text-2xl">Page introuvable</Heading>
  <P>Cette adresse ne correspond à aucune page.</P>
  <A href="/" onclick={follow}>← Personnages</A>
</section>
