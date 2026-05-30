import { tagColor, useColorMaps } from "./lib/colors";

interface Props {
  tag: string;
  // When provided, the pill is interactive (cursor pointer + hover dim) and
  // calls onClick. Without it, the pill is purely visual (palette / preview).
  onClick?: () => void;
}

export function TagPill({ tag, onClick }: Props) {
  const maps = useColorMaps();
  const color = tagColor(tag, maps);
  const interactive = onClick !== undefined;
  return (
    <span
      onClick={onClick}
      style={{ backgroundColor: color }}
      className={
        "inline-block rounded-full px-2 py-0.5 text-xs text-white transition-colors duration-200 " +
        (interactive
          ? "cursor-pointer hover:opacity-80"
          : "cursor-default")
      }
    >
      {tag}
    </span>
  );
}
