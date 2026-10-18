<script lang="ts">
  // The colour theme of this device (ADR FE-THEMES).
  import { valueOf } from "./character/edits";
  import { THEMES, setTheme, theme } from "./theme.svelte";
</script>

<label class="theme">
  Thème
  <select
    value={theme.choice}
    onchange={(e) => {
      const next = THEMES.find((t) => t.id === valueOf(e));
      if (next) setTheme(next.id);
    }}
  >
    {#each THEMES as t (t.id)}
      <option value={t.id}>{t.name}</option>
    {/each}
  </select>
</label>

<style>
  .theme {
    display: inline-flex;
    gap: 0.5rem;
    align-items: center;
  }

  select {
    flex: none;
  }
</style>
