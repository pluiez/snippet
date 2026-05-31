// Frontend mirror of src-tauri/src/render.rs::render. Used for live preview
// in the fill dialog without IPC roundtrips. The regex and substitution rule
// must stay in sync with the Rust side; SPEC §13 invariant 10 is the contract.

const PLACEHOLDER_RE =
  /\{([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\}/g;

export function render(body: string, values: Record<string, string>): string {
  return body.replace(PLACEHOLDER_RE, (_match, guid) => {
    const value = values[guid];
    return value ?? "";
  });
}
