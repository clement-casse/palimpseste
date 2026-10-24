import type { Engine, Sheet, StepStatus } from "./wasm/palimpseste_wasm";

export const ENGINES: Record<Engine, string> = {
  fantasy: "Chroniques Oubliées Fantasy",
  contemporain: "Chroniques Oubliées Contemporain",
};

/**
 * What is left to settle over every creation step: missing choices and
 * broken rules no waiver accepts. None means the character is finished.
 */
export function unsettled(steps: StepStatus[]): number {
  return steps.reduce((sum, step) => sum + step.incomplete + step.deviations, 0);
}

/**
 * What is left to settle over the creation and every level-up or spending
 * of XP. None means the character is complete: the next level-up or
 * spending may start (ADR FE-PROGRESSION).
 */
export function unsettledAll(sheet: Sheet): number {
  return (
    unsettled(sheet.steps) +
    sheet.progression.reduce((sum, entry) => sum + unsettled(entry.steps), 0)
  );
}
