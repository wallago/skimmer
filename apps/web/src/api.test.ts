import { afterEach, expect, test, vi } from "vitest";
import { type Day, formatCost, formatTokens, setStatus, unreadCount } from "./api";

afterEach(() => {
  vi.unstubAllGlobals();
});

/** A day with the given counts and no spend. */
function day(unread: number, viewed: number, finished: number): Day {
  return {
    day: "2026-11-23",
    unread,
    viewed,
    finished,
    input_tokens: 0,
    output_tokens: 0,
    cost_usd: null,
  };
}

test("counts briefings not finished yet", () => {
  expect(unreadCount([day(2, 1, 4), day(0, 1, 0)])).toBe(4);
  expect(unreadCount([])).toBe(0);
});

test("formats cost and tokens", () => {
  expect(formatCost(0.01234)).toBe("$0.0123");
  expect(formatCost(null)).toBe("—");
  expect(formatTokens(999)).toBe("999");
  expect(formatTokens(12_345)).toBe("12.3k");
});

test("setStatus sends a PUT with the new status", async () => {
  const fetch = vi.fn(async () => new Response(JSON.stringify({ id: 3, status: "finished" })));
  vi.stubGlobal("fetch", fetch);
  await setStatus(3, "finished");
  expect(fetch).toHaveBeenCalledWith(
    "/api/briefings/3/status",
    expect.objectContaining({ method: "PUT", body: '{"status":"finished"}' }),
  );
});

test("a failed request throws", async () => {
  vi.stubGlobal("fetch", async () => new Response("{}", { status: 404 }));
  await expect(setStatus(9, "viewed")).rejects.toThrow("404");
});
