<script lang="ts">
  // What every visitor should know about Palimpseste itself, on every page
  // (NOTICE.md): a fan project, unfinished, and where its source is.
  import { Footer, FooterLink, FooterLinkGroup } from "flowbite-svelte";

  const REPOSITORY = "https://github.com/clement-casse/palimpseste";

  // As wide as the page above it.
  let { wide = false }: { wide?: boolean } = $props();
</script>

<Footer
  footerType="sitemap"
  class={[
    "mx-auto mt-4 space-y-1 border-t border-gray-200 bg-transparent px-4 py-3 text-sm text-gray-500 dark:border-gray-700 dark:bg-transparent dark:text-gray-400",
    wide ? "max-w-5xl" : "max-w-2xl",
  ]}
>
  <p>
    <strong>Palimpseste est un projet de fan, non officiel.</strong> Il n’est pas affilié à
    Black Book Éditions ni à la gamme <cite>Chroniques Oubliées</cite>, dont les règles, les noms
    et les données appartiennent à Black Book Éditions et à leurs auteurs.
  </p>
  <p>
    <strong>Palimpseste est en cours de développement et fourni sans aucune garantie.</strong>
    Les données peuvent être effacées et les inscriptions fermées à tout moment.
  </p>
  <FooterLinkGroup class="flex flex-wrap gap-4">
    <FooterLink href={REPOSITORY}>Code source</FooterLink>
    <FooterLink href={`${REPOSITORY}/blob/main/NOTICE.md`}>Avertissement complet</FooterLink>
    <li>Licence AGPL-3.0</li>
  </FooterLinkGroup>
</Footer>
