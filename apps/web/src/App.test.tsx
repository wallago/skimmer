import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { App } from "./App";
import type { Briefing, Day } from "./api";

const DAY: Day = {
  day: "2026-11-23",
  unread: 1,
  viewed: 0,
  finished: 0,
  input_tokens: 1500,
  output_tokens: 300,
  cost_usd: 0.0003,
};

const BRIEFING: Briefing = {
  id: 1,
  topic: "rust",
  day: "2026-11-23",
  question: "What's new?",
  since: "2026-11-22T06:00:00Z",
  scanned: 1,
  digest: "A quiet day.",
  highlights: [{ headline: "Rust 2.0", detail: "Not really.", entry_ids: [42] }],
  sources: [{ id: 42, title: "T", feed: "This Week in Rust", url: "https://example.com/42" }],
  model: "claude-haiku-5-5",
  status: "unread",
  updated_at: "2026-11-23T06:00:00Z",
};

/** Answers the API from the fixtures above; records status changes. */
let puts: string[] = [];

beforeEach(() => {
  puts = [];
  try {
    localStorage.clear();
  } catch {
    // No storage in this environment.
  }
  vi.stubGlobal("fetch", async (url: string, init?: RequestInit) => {
    if (init?.method === "PUT") {
      puts.push(`${url} ${init.body}`);
      const { status } = JSON.parse(String(init.body)) as { status: Briefing["status"] };
      return new Response(JSON.stringify({ ...BRIEFING, status }));
    }
    if (url.endsWith("api/days")) return new Response(JSON.stringify([DAY]));
    return new Response(JSON.stringify([BRIEFING]));
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
});

test("shows the morning popup, then the day's briefings", async () => {
  render(<App />);
  expect(await screen.findByRole("dialog", { name: "New briefings" })).toBeTruthy();
  fireEvent.click(screen.getByText("Later"));
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(await screen.findByText("Rust 2.0")).toBeTruthy();
});

test("opening a briefing marks it viewed and shows its highlights", async () => {
  render(<App />);
  fireEvent.click(await screen.findByText("Later"));
  const card = (await screen.findByText("Rust 2.0")).closest(".card");
  if (!card) throw new Error("no card");
  fireEvent.pointerDown(card, { clientX: 10, clientY: 10 });
  fireEvent.pointerUp(card, { clientX: 10, clientY: 10 });
  expect(await screen.findByText("Not really.")).toBeTruthy();
  expect(puts).toEqual(['/api/briefings/1/status {"status":"viewed"}']);
});

test("swiping right marks it read, with an undo", async () => {
  render(<App />);
  fireEvent.click(await screen.findByText("Later"));
  const card = (await screen.findByText("Rust 2.0")).closest(".card");
  if (!card) throw new Error("no card");
  fireEvent.pointerDown(card, { clientX: 10, clientY: 10 });
  fireEvent.pointerUp(card, { clientX: 200, clientY: 12 });
  expect(await screen.findByText("Marked as read")).toBeTruthy();
  expect(puts).toEqual(['/api/briefings/1/status {"status":"finished"}']);
});

test("the usage page lists tokens and cost per day", async () => {
  render(<App />);
  fireEvent.click(await screen.findByText("Later"));
  fireEvent.click(screen.getByText("Usage"));
  expect(screen.getByText("1.5k")).toBeTruthy();
  expect(screen.getAllByText("$0.0003").length).toBeGreaterThan(0);
});

test("a new morning arriving while the app is open shows the popup", async () => {
  // The first load: yesterday, already read, so no popup.
  const read: Day = { ...DAY, unread: 0, finished: 1 };
  let days = [read];
  vi.stubGlobal("fetch", async (url: string) =>
    url.endsWith("api/days")
      ? new Response(JSON.stringify(days))
      : new Response(JSON.stringify([{ ...BRIEFING, status: "finished" }])),
  );
  render(<App />);
  expect(await screen.findByText("Rust 2.0")).toBeTruthy();
  expect(screen.queryByRole("dialog")).toBeNull();
  // 07:00: a new day lands, and the app is brought back on screen.
  days = [{ ...DAY, day: "2026-11-24" }, read];
  fireEvent(document, new Event("visibilitychange"));
  expect(await screen.findByRole("dialog", { name: "New briefings" })).toBeTruthy();
});
