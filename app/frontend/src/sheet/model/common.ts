// What both engines' sheets show, as plain data: the view-models
// (`fantasy.ts`, `contemporain.ts`) shape what the engine computed, and the
// components only lay it out.

import type { ContributionLine, ItemEntry, SituationalLine, ValueLine } from "../../lib/engine";
import { shown } from "../values";

/** A labelled box of the identity. */
export type Field = { label: string; value: string };

/** A value shown as a tile: its short label, and the total as written. */
export type Tile = {
  /** The identifier of the value: its notes are placed by it. */
  id: string;
  label: string;
  /** The full name, when the label is an abbreviation. */
  title?: string;
  /** The total, signed for a modifier; a dash while it cannot be computed. */
  text: string;
  contributions?: ContributionLine[];
  situational?: SituationalLine[];
  /** The roll the tile opens, when it carries one. */
  roll?: string;
};

/** The tile of `line`, a dash when the engine lists no such value. */
export function tile(
  id: string,
  line: ValueLine | undefined,
  { label, title, signed = false }: { label: string; title?: string; signed?: boolean },
): Tile {
  return {
    id,
    label,
    title,
    text: shown(line, signed),
    contributions: line?.contributions,
    situational: line?.situational,
    roll: line?.roll ?? undefined,
  };
}

/** How many of weapon `id` are among `items`. */
export function carried(items: ItemEntry[] | undefined, id: string): number {
  return (items ?? [])
    .filter((item) => item.kind === "weapon" && item.id === id)
    .reduce((sum, item) => sum + item.quantity, 0);
}
