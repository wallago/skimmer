// The `skimmer serve` API: days with their counts and spend, a day's
// briefings, and setting a briefing's read state.

/** How far the reader got through a briefing. */
export type Status = "unread" | "viewed" | "finished";

/** One day with briefings: counts per state, and what they cost to write. */
export interface Day {
  day: string;
  unread: number;
  viewed: number;
  finished: number;
  input_tokens: number;
  output_tokens: number;
  /** USD, `null` when the model had no known price. */
  cost_usd: number | null;
}

/** One highlight, citing the entries it came from. */
export interface Highlight {
  headline: string;
  detail: string;
  entry_ids: number[];
}

/** One entry Claude read. */
export interface Source {
  id: number;
  title: string;
  feed: string;
  url: string;
}

/** One topic's briefing for one day. */
export interface Briefing {
  id: number;
  topic: string;
  day: string;
  question: string;
  since: string;
  scanned: number;
  digest: string;
  highlights: Highlight[];
  sources: Source[];
  model: string;
  status: Status;
  updated_at: string;
}

/** Fetches `path` as JSON, throwing on a non-2xx answer. */
async function json<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${import.meta.env.BASE_URL}${path}`, init);
  if (!response.ok) {
    throw new Error(`${init?.method ?? "GET"} ${path}: ${response.status}`);
  }
  return (await response.json()) as T;
}

/** Every day with briefings, newest first. */
export function getDays(): Promise<Day[]> {
  return json<Day[]>("api/days");
}

/** The briefings of `day` (`YYYY-MM-DD`). */
export function getDay(day: string): Promise<Briefing[]> {
  return json<Briefing[]>(`api/days/${day}`);
}

/** Sets a briefing's read state; answers the briefing as saved. */
export function setStatus(id: number, status: Status): Promise<Briefing> {
  return json<Briefing>(`api/briefings/${id}/status`, {
    method: "PUT",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ status }),
  });
}

/** Briefings not finished yet, across `days`: the number on the app icon. */
export function unreadCount(days: readonly Day[]): number {
  return days.reduce((sum, d) => sum + d.unread + d.viewed, 0);
}

/** `$0.0123`, or `—` when the price is unknown. */
export function formatCost(cost: number | null): string {
  return cost === null ? "—" : `$${cost.toFixed(4)}`;
}

/** `12.3k` for counts from a thousand up. */
export function formatTokens(tokens: number): string {
  return tokens < 1000 ? `${tokens}` : `${(tokens / 1000).toFixed(1)}k`;
}
