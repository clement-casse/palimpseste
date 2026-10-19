<script lang="ts">
  // The colour theme of this device (ADR FE-THEMES).
  import { Label, Select } from "flowbite-svelte";

  import { valueOf } from "./character/edits";
  import { THEMES, setTheme, theme } from "./theme.svelte";

  const items = THEMES.map((t) => ({ value: t.id, name: t.name }));
</script>

<Label class="inline-flex items-center gap-2">
  Thème
  <Select
    placeholder=""
    size="sm"
    class="w-auto"
    {items}
    value={theme.choice}
    onchange={(e) => {
      const next = THEMES.find((t) => t.id === valueOf(e));
      if (next) setTheme(next.id);
    }}
  />
</Label>
