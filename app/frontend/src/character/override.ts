import type { ReportedViolation, Target } from "../wasm/palimpseste_wasm";

/** What the Game Master is about to override. */
export type OverrideRequest =
  | { kind: "waiver"; violation: ReportedViolation }
  | { kind: "adjustment"; target: Target; label: string };
