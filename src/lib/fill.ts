// Helpers for the fill ↔ edit round-trip (SPEC §4.7).
//
// When the user comes back from edit mode to fill mode, variables may have
// been added / removed / renamed / had their type or options changed. Filled
// values are preserved by GUID (SPEC §13 invariant 1) when they are still
// valid for the variable; otherwise they fall through to whatever cascade
// initial value `prepare_fill_dialog` produced.

import type { FillDialogState } from "./bindings/FillDialogState";
import type { Variable } from "./bindings/Variable";

export function mergeFillValues(
  newState: FillDialogState,
  oldValues: Record<string, string>
): Record<string, string> {
  const merged: Record<string, string> = {};
  for (const v of newState.orderedVariables) {
    const oldVal = oldValues[v.guid];
    if (oldVal !== undefined && isValidForVariable(v, oldVal)) {
      merged[v.guid] = oldVal;
    } else {
      merged[v.guid] = newState.initialValues[v.guid] ?? "";
    }
  }
  return merged;
}

function isValidForVariable(v: Variable, value: string): boolean {
  if (v.type === "text") return true;
  return (v.options ?? []).includes(value);
}
