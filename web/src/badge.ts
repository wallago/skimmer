// The number on the installed app's icon: briefings not finished yet.

/** What badging needs of `navigator`: absent where the browser has no badges. */
export type Badging = Partial<Pick<Navigator, "setAppBadge" | "clearAppBadge">>;

/** Shows `count` on the app icon, or clears it at 0. Never throws. */
export async function showBadge(count: number, nav: Badging = navigator): Promise<void> {
  try {
    if (count > 0) await nav.setAppBadge?.(count);
    else await nav.clearAppBadge?.();
  } catch {
    // Not installed, or badges turned off: the icon just shows no number.
  }
}
