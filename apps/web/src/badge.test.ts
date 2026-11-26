import { expect, test, vi } from "vitest";
import type { Briefing, Status } from "./api";
import { type Badging, showBadge, showUnread } from "./badge";

function briefing(id: number, status: Status = "unread", topic = "rust"): Briefing {
  return {
    id,
    topic,
    day: "2026-11-25",
    question: "What's new?",
    since: "2026-11-24T06:00:00Z",
    scanned: 3,
    digest: "A quiet day.",
    highlights: [{ headline: `Headline ${id}`, detail: "", entry_ids: [] }],
    sources: [],
    model: "claude-haiku-5-5",
    status,
    updated_at: "2026-11-25T06:00:00Z",
  };
}

function badging() {
  return {
    setAppBadge: vi.fn(() => Promise.resolve()),
    clearAppBadge: vi.fn(() => Promise.resolve()),
  } satisfies Badging;
}

test("shows the count on the icon", async () => {
  const nav = badging();
  await showBadge(4, nav);
  expect(nav.setAppBadge).toHaveBeenCalledWith(4);
});

test("clears the icon when nothing is left", async () => {
  const nav = badging();
  await showBadge(0, nav);
  expect(nav.clearAppBadge).toHaveBeenCalled();
  expect(nav.setAppBadge).not.toHaveBeenCalled();
});

test("skips a browser without badges", async () => {
  await expect(showBadge(2, {})).resolves.toBeUndefined();
});

/** A fake registration holding its notifications by tag. */
function notifier(tags: string[] = []) {
  const shown = new Map<string, { title: string; body: string; close: () => void }>();
  for (const tag of tags)
    shown.set(tag, { title: "old", body: "old", close: () => shown.delete(tag) });
  return {
    shown,
    getNotifications: vi.fn(async () =>
      [...shown.entries()].map(([tag, n]) => ({
        tag,
        title: n.title,
        body: n.body,
        close: n.close,
      })),
    ),
    showNotification: vi.fn(async (title: string, options?: NotificationOptions) => {
      const tag = options?.tag ?? "";
      shown.set(tag, { title, body: options?.body ?? "", close: () => shown.delete(tag) });
    }),
  };
}

test("posts one quiet notification per unread briefing", async () => {
  const reg = notifier();
  await showUnread([briefing(1), briefing(2, "unread", "dofus"), briefing(3, "viewed")], reg);
  expect([...reg.shown.keys()].sort()).toEqual(["briefing:1", "briefing:2"]);
  expect(reg.shown.get("briefing:2")).toMatchObject({ title: "dofus", body: "Headline 2" });
  expect(reg.showNotification).toHaveBeenCalledWith(
    "rust",
    expect.objectContaining({ silent: true }),
  );
});

test("closes the notification of an opened briefing, and leaves others' alone", async () => {
  const reg = notifier(["briefing:1", "briefing:2", "due:other"]);
  reg.shown.set("briefing:1", { title: "rust", body: "Headline 1", close: () => {} });
  await showUnread([briefing(1), briefing(2, "finished")], reg);
  expect([...reg.shown.keys()].sort()).toEqual(["briefing:1", "due:other"]);
  // Unchanged: not posted again, so it doesn't move or buzz.
  expect(reg.showNotification).not.toHaveBeenCalled();
});

test("skips notifications without a registration or permission", async () => {
  await expect(showUnread([briefing(1)], undefined)).resolves.toBeUndefined();
  const reg = notifier();
  reg.showNotification.mockRejectedValue(new TypeError("no permission"));
  await expect(showUnread([briefing(1)], reg)).resolves.toBeUndefined();
});
