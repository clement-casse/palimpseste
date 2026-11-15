// The engines, compiled to WebAssembly (ADR APP-BINDINGS): the rules run in
// the browser. This module is the only one to import the generated bindings
// (`src/wasm/`); everything else reads the engines, and their types, here.

import init, { initSync } from "../../wasm/palimpseste_wasm";
import type {
  Catalog,
  ContemporainCatalog,
  ContemporainDetails,
  ContemporainDocument,
  ContemporainEquipment,
  Document,
  Equipment,
  FantasyCatalog,
  FantasyDetails,
  FantasyDocument,
  FantasyEquipment,
  Sheet,
} from "../../wasm/palimpseste_wasm";

export type * from "../../wasm/palimpseste_wasm";
// These throw what the engine refuses (a document it cannot read, a roll it
// does not know), as an `Error`: call them through `attempt` wherever the
// input comes from storage or the backend.
export {
  catalog,
  emptyDocument,
  evaluate,
  evaluateInPlay,
  roll,
  rollOffer,
} from "../../wasm/palimpseste_wasm";

/** Loads the engines: once, before anything calls them. */
export async function loadEngines(): Promise<void> {
  await init();
}

/** Loads the engines from the bytes of their module, for the tests. */
export function loadEnginesSync(module: BufferSource): void {
  initSync({ module });
}

/** What a call gave, or why the engine refused it. */
export type Result<T> = { ok: true; value: T } | { ok: false; error: string };

/** Runs `call`, a call to the engines, giving its refusal in place of throwing. */
export function attempt<T>(call: () => T): Result<T> {
  try {
    return { ok: true, value: call() };
  } catch (error) {
    return { ok: false, error: error instanceof Error ? error.message : String(error) };
  }
}

/** A character of one engine: its document, catalogue and details, narrowed together. */
export type EngineView =
  | {
      engine: "fantasy";
      choices: { engine: "fantasy" } & FantasyDocument;
      catalog: { engine: "fantasy" } & FantasyCatalog;
      sheet: Sheet;
      details: { engine: "fantasy" } & FantasyDetails;
    }
  | {
      engine: "contemporain";
      choices: { engine: "contemporain" } & ContemporainDocument;
      catalog: { engine: "contemporain" } & ContemporainCatalog;
      sheet: Sheet;
      details: { engine: "contemporain" } & ContemporainDetails;
    };

/**
 * The document, catalogue and details of one engine, so nothing is cast;
 * `null` when they are not of the same engine.
 */
export function narrow(choices: Document, catalog: Catalog, sheet: Sheet): EngineView | null {
  const details = sheet.details;
  if (
    choices.engine === "fantasy" &&
    catalog.engine === "fantasy" &&
    details.engine === "fantasy"
  ) {
    return { engine: "fantasy", choices, catalog, sheet, details };
  }
  if (
    choices.engine === "contemporain" &&
    catalog.engine === "contemporain" &&
    details.engine === "contemporain"
  ) {
    return { engine: "contemporain", choices, catalog, sheet, details };
  }
  return null;
}

/** A character in play: an `EngineView` with the equipment of its engine. */
export type PlayView =
  | (Extract<EngineView, { engine: "fantasy" }> & {
      equipment: { engine: "fantasy" } & FantasyEquipment;
    })
  | (Extract<EngineView, { engine: "contemporain" }> & {
      equipment: { engine: "contemporain" } & ContemporainEquipment;
    });

/** `view` with `equipment`, `null` when it is another engine's. */
export function withEquipment(view: EngineView, equipment: Equipment): PlayView | null {
  if (view.engine === "fantasy" && equipment.engine === "fantasy") return { ...view, equipment };
  if (view.engine === "contemporain" && equipment.engine === "contemporain") {
    return { ...view, equipment };
  }
  return null;
}
