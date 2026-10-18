// The equipment a character wears and carries in play (ADR ENG-IN-PLAY), as
// `evaluateInPlay` reads it. Only fields are picked here: the engines apply
// every rule.

import type { Document, Engine, Equipment } from "../wasm/palimpseste_wasm";

/** The equipment chosen at creation, as an equipment in play. */
export function equipmentOf(document: Document): Equipment {
  return document.engine === "fantasy"
    ? {
        engine: "fantasy",
        armour: document.armour,
        shield: document.shield,
        items: document.items ?? [],
      }
    : {
        engine: "contemporain",
        protections: document.protections ?? [],
        items: document.items ?? [],
      };
}

/**
 * Whether `kept` has the shape of an equipment of `engine`. Its content is
 * read by the engine, which rejects an identifier it does not know.
 */
export function isEquipment(kept: unknown, engine: Engine): kept is Equipment {
  return (
    typeof kept === "object" &&
    kept !== null &&
    "engine" in kept &&
    kept.engine === engine &&
    "items" in kept &&
    Array.isArray(kept.items)
  );
}
