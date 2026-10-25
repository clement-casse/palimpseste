<script lang="ts">
  // The terms of a value, shown while the element `triggeredBy` names is
  // hovered or focused: every source of a characteristic, a capacité
  // included, read without opening anything.
  import { Popover } from "flowbite-svelte";

  import type { ContributionLine, SituationalLine } from "../wasm/palimpseste_wasm";
  import Breakdown from "./Breakdown.svelte";

  let {
    triggeredBy,
    title,
    contributions,
    situational = [],
  }: {
    triggeredBy: string;
    title: string;
    contributions: ContributionLine[];
    situational?: SituationalLine[];
  } = $props();
</script>

<Popover {triggeredBy} {title} trigger="hover" class="w-64 text-sm">
  <Breakdown {contributions} {situational} />
</Popover>
