// Renders a template body with `{<guid>}` placeholders replaced by inline
// "chip" elements colored from variableColorMap (SPEC §4.3 / §9.4: square
// corners, no hover, distinct from tag pills' rounded-full pill).
//
// Used by the palette preview where variables are *unfilled* — the user is
// previewing the template, not the rendered output.

import { useMemo } from "react";
import type { Variable } from "./lib/bindings/Variable";
import { useColorMaps, variableColor } from "./lib/colors";

const PLACEHOLDER_RE =
  /\{([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\}/g;

interface Props {
  body: string;
  variables: Variable[];
  className?: string;
}

type Part =
  | { type: "text"; content: string }
  | { type: "var"; name: string };

export function BodyWithVariableChips({ body, variables, className }: Props) {
  const maps = useColorMaps();
  const parts = useMemo(() => parseBody(body, variables), [body, variables]);

  if (parts.length === 0) {
    return (
      <pre className={className}>
        <span className="italic text-zinc-400">（空）</span>
      </pre>
    );
  }

  return (
    <pre
      className={
        "whitespace-pre-wrap break-words font-mono text-sm text-zinc-900 " +
        (className ?? "")
      }
    >
      {parts.map((p, i) =>
        p.type === "text" ? (
          <span key={i}>{p.content}</span>
        ) : (
          <span
            key={i}
            style={{ backgroundColor: variableColor(p.name, maps) }}
            className="inline-block rounded-sm px-1.5 align-baseline font-mono text-xs text-white"
          >
            {p.name}
          </span>
        )
      )}
    </pre>
  );
}

function parseBody(body: string, variables: Variable[]): Part[] {
  const guidToName = new Map(variables.map((v) => [v.guid, v.displayName]));
  const parts: Part[] = [];
  let lastEnd = 0;
  for (const match of body.matchAll(PLACEHOLDER_RE)) {
    const start = match.index ?? 0;
    const end = start + match[0].length;
    if (start > lastEnd) {
      parts.push({ type: "text", content: body.slice(lastEnd, start) });
    }
    const guid = match[1];
    const name = guidToName.get(guid);
    if (name) {
      parts.push({ type: "var", name });
    } else {
      // Orphan placeholder: keep the literal `{<guid>}` text.
      parts.push({ type: "text", content: match[0] });
    }
    lastEnd = end;
  }
  if (lastEnd < body.length) {
    parts.push({ type: "text", content: body.slice(lastEnd) });
  }
  return parts;
}
