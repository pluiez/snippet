// Shared animation constants for framer-motion and CSS animations.
// All durations in seconds (framer-motion convention).

export const DURATION = {
  fast: 0.15, // toast, dialog enter/exit
  normal: 0.2, // view transitions, palette fade
  slow: 0.3, // larger area transitions
} as const;

export const EASE = {
  out: [0.16, 1, 0.3, 1] as const, // enter
  in: [0.4, 0, 1, 1] as const, // exit
  inOut: [0.4, 0, 0.2, 1] as const, // bidirectional
};
