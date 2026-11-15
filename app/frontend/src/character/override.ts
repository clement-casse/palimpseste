import type { ReportedViolation, Target } from "../lib/engine";

/** What the Game Master is about to override. */
export type OverrideRequest =
  | { kind: "waiver"; violation: ReportedViolation }
  | { kind: "adjustment"; target: Target; label: string };
