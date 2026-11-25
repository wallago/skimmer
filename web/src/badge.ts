// The number on the installed app's icon: briefings not finished yet.
// Desktop Chrome and Edge take the number through the Badging API. Android
// has no such API: its launcher badges an app by its notifications, so each
// new briefing also gets a quiet notification, as wallago/todo does for its
// due todos.

import type { Briefing } from "./api";

/** Tag prefix of the briefing notifications, so other notifications are left alone. */
const TAG = "briefing:";

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

/** A notification as `getNotifications` gives it back. */
interface Shown {
  tag: string;
  title: string;
  body: string;
  close(): void;
}

/** What notifying needs of the service worker's registration. */
export interface Notifier {
  getNotifications(): Promise<Shown[]>;
  showNotification(title: string, options?: NotificationOptions): Promise<void>;
}

/** The notification's second line: the first headline, else the question. */
function bodyOf(briefing: Briefing): string {
  return briefing.highlights[0]?.headline ?? briefing.question;
}

/**
 * One quiet notification per unread briefing, closed once it is opened or
 * read. An unchanged one is left as is, so it doesn't move or buzz. Does
 * nothing without a registration or notification permission. Never throws.
 */
export async function showUnread(
  briefings: readonly Briefing[],
  reg: Notifier | undefined,
): Promise<void> {
  if (!reg) return;
  try {
    const unread = new Map(
      briefings.filter((b) => b.status === "unread").map((b) => [`${TAG}${b.id}`, b]),
    );
    const shown = (await reg.getNotifications()).filter((n) => n.tag.startsWith(TAG));
    for (const n of shown) {
      if (!unread.has(n.tag)) n.close();
    }
    for (const [tag, briefing] of unread) {
      const body = bodyOf(briefing);
      const old = shown.find((n) => n.tag === tag);
      if (old?.title === briefing.topic && old.body === body) continue;
      await reg.showNotification(briefing.topic, {
        tag,
        body,
        silent: true,
        icon: `${import.meta.env.BASE_URL}icons/icon-192.png`,
        data: { id: briefing.id },
      });
    }
  } catch {
    // No permission, or notifications turned off: the icon shows no badge.
  }
}

/** The worker's registration once notifications are allowed, else nothing. */
export async function notifier(): Promise<Notifier | undefined> {
  if (typeof Notification === "undefined" || Notification.permission !== "granted")
    return undefined;
  if (!("serviceWorker" in navigator)) return undefined;
  return navigator.serviceWorker.getRegistration();
}

/** Puts the count on the icon, both ways: `newest` is the newest day's briefings. */
export async function updateBadge(count: number, newest: readonly Briefing[]): Promise<void> {
  await showBadge(count);
  await showUnread(newest, await notifier());
}
