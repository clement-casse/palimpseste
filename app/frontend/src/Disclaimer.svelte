<script lang="ts">
  // What every visitor should know about Palimpseste itself, on every page
  // (NOTICE.md): a fan project, unfinished, and where its source is.
  const REPOSITORY = "https://github.com/clement-casse/palimpseste";

  // As wide as the page above it.
  let { wide = false }: { wide?: boolean } = $props();
</script>

<footer class={{ wide }}>
  <p>
    <strong>Palimpseste est un projet de fan, non officiel.</strong> Il n’est pas affilié à
    Black Book Éditions ni à la gamme <cite>Chroniques Oubliées</cite>, dont les règles, les noms
    et les données appartiennent à Black Book Éditions et à leurs auteurs.
  </p>
  <p>
    <strong>Palimpseste est en cours de développement et fourni sans aucune garantie.</strong>
    Les données peuvent être effacées et les inscriptions fermées à tout moment.
  </p>
  <p>
    <a href={REPOSITORY}>Code source</a> sous licence AGPL-3.0 ·
    <a href={`${REPOSITORY}/blob/main/NOTICE.md`}>Avertissement complet</a>
  </p>
</footer>

<style>
  footer {
    max-width: 40rem;
    margin: 1rem auto;
    padding: 0.75rem 1rem 0;
    border-top: 1px solid var(--muted);
    color: var(--muted);
    font-size: 0.875rem;
  }

  footer.wide {
    max-width: 64rem;
  }

  p {
    margin: 0.25rem 0;
  }
</style>
