// Gmail-style swipes on a briefing row.

/** How far a finger has to travel sideways to count as a swipe. */
export const MIN_SWIPE_PX = 80;

/** What a finished drag does: right marks it read, left keeps it for later. */
export type SwipeAction = "read" | "later";

/**
 * The action for a drag of `dx` by `dy` pixels, or `null` for a tap, a short
 * drag or a mostly-vertical move (a scroll).
 */
export function swipeAction(dx: number, dy: number): SwipeAction | null {
  if (Math.abs(dx) < MIN_SWIPE_PX || Math.abs(dx) <= 2 * Math.abs(dy)) return null;
  return dx > 0 ? "read" : "later";
}
