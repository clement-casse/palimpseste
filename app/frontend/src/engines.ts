import type { Engine, StepStatus } from "./wasm/palimpseste_wasm";

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
