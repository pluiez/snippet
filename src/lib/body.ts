// Bidirectional body transform between storage form (`{<guid>}`) and editor
// display form (`{<displayName>}`). SPEC §2.2: bodies are stored with GUID
// placeholders so renaming a displayName doesn't lose filled values; the user
// sees the displayName form so the body is actually readable.
//
// Round-trip rules:
//   - bodyToDisplay: storage `{<guid>}` → display `{<displayName>}` when the
//     GUID matches a variable. Unknown GUIDs are kept as-is (orphan placeholder).
//   - bodyToStorage: display `{<displayName>}` → storage `{<guid>}` when the
//     name matches a variable. Unmatched `{...}` content is kept as literal
//     text (the user typed something that's not a variable reference). UUID
//     literals like `{a1b2...-...}` are preserved as-is so a round-trip on a
//     template with an orphan placeholder doesn't fabricate a new variable.

import type { Variable } from "./bindings/Variable";

const UUID_PATTERN =
  /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/;

const GUID_PLACEHOLDER_RE =
  /\{([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\}/g;

const DISPLAY_PLACEHOLDER_RE = /\{([^{}]+)\}/g;

export function bodyToDisplay(body: string, variables: Variable[]): string {
  return body.replace(GUID_PLACEHOLDER_RE, (match, guid) => {
    const v = variables.find((v) => v.guid === guid);
    return v ? `{${v.displayName}}` : match;
  });
}

export function bodyToStorage(
  displayBody: string,
  variables: Variable[]
): string {
  return displayBody.replace(DISPLAY_PLACEHOLDER_RE, (match, content) => {
    if (UUID_PATTERN.test(content)) {
      // Already a UUID literal; let it through unchanged.
      return match;
    }
    const v = variables.find((v) => v.displayName === content);
    return v ? `{${v.guid}}` : match;
  });
}
