import { useCallback, useEffect, useState } from "react";
import {
  type Briefing,
  type Day,
  getDay,
  getDays,
  type Status,
  setStatus,
  unreadCount,
} from "./api";
import { showBadge } from "./badge";
import { BriefingRow } from "./components/BriefingRow";
import { Reader } from "./components/Reader";
import { Usage } from "./components/Usage";
import type { SwipeAction } from "./swipe";

/** Where the morning popup remembers the last day it was shown for. */
const POPUP_KEY = "skimmer.popup-day";

/** The day the popup was last shown for, if the browser lets us know. */
function popupDay(): string | null {
  try {
    return localStorage.getItem(POPUP_KEY);
  } catch {
    return null;
  }
}

/** Remembers that the popup was shown for `day`. */
function rememberPopup(day: string) {
  try {
    localStorage.setItem(POPUP_KEY, day);
  } catch {
    // Private window: the popup may show again, which is harmless.
  }
}

/** The last status change, so the toast can undo it. */
interface Undo {
  id: number;
  status: Status;
  text: string;
}

/** The app: the day's briefings, a reader, and the usage history. */
export function App() {
  const [page, setPage] = useState<"today" | "usage">("today");
  const [days, setDays] = useState<Day[]>([]);
  const [day, setDay] = useState<string | null>(null);
  const [briefings, setBriefings] = useState<Briefing[]>([]);
  const [open, setOpen] = useState<number | null>(null);
  const [popup, setPopup] = useState(false);
  const [undo, setUndo] = useState<Undo | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refreshDays = useCallback(async () => {
    const fresh = await getDays();
    setDays(fresh);
    void showBadge(unreadCount(fresh));
    return fresh;
  }, []);

  useEffect(() => {
    refreshDays()
      .then((fresh) => {
        const newest = fresh[0];
        if (!newest) return;
        setDay(newest.day);
        if (newest.unread > 0 && popupDay() !== newest.day) setPopup(true);
      })
      .catch((e: unknown) => setError(String(e)));
  }, [refreshDays]);

  useEffect(() => {
    if (!undo) return;
    const timer = setTimeout(() => setUndo(null), 4000);
    return () => clearTimeout(timer);
  }, [undo]);

  useEffect(() => {
    if (day) getDay(day).then(setBriefings, (e: unknown) => setError(String(e)));
  }, [day]);

  const change = useCallback(
    async (id: number, status: Status, text?: string) => {
      const before = briefings.find((b) => b.id === id);
      if (!before || before.status === status) return;
      try {
        const saved = await setStatus(id, status);
        setBriefings((list) => list.map((b) => (b.id === id ? saved : b)));
        if (text) setUndo({ id, status: before.status, text });
        await refreshDays();
      } catch (e) {
        setError(String(e));
      }
    },
    [briefings, refreshDays],
  );

  const openBriefing = (b: Briefing) => {
    setOpen(b.id);
    if (b.status === "unread") void change(b.id, "viewed");
  };

  const swipe = (b: Briefing, action: SwipeAction) =>
    action === "read"
      ? change(b.id, "finished", "Marked as read")
      : change(b.id, "unread", "Kept for later");

  const closePopup = () => {
    setPopup(false);
    if (day) rememberPopup(day);
  };

  const current = briefings.find((b) => b.id === open);
  const finishCurrent = useCallback(() => {
    if (open !== null) void change(open, "finished");
  }, [open, change]);

  if (current) {
    return (
      <main>
        <Reader briefing={current} onBack={() => setOpen(null)} onFinished={finishCurrent} />
      </main>
    );
  }

  const today = days.find((d) => d.day === day);
  const left = briefings.filter((b) => b.status !== "finished");
  const order = [...left, ...briefings.filter((b) => b.status === "finished")];

  return (
    <>
      <header>
        <b>skimmer</b>
        {unreadCount(days) > 0 && <span className="badge">{unreadCount(days)}</span>}
        <nav>
          <button
            type="button"
            className={page === "today" ? "active" : ""}
            onClick={() => setPage("today")}
          >
            Today
          </button>
          <button
            type="button"
            className={page === "usage" ? "active" : ""}
            onClick={() => setPage("usage")}
          >
            Usage
          </button>
        </nav>
      </header>
      <main>
        {error && <p className="status error">{error}</p>}
        {page === "usage" ? (
          <Usage days={days} />
        ) : (
          <>
            <div className="days">
              {days.map((d) => (
                <button
                  type="button"
                  key={d.day}
                  className={d.day === day ? "day on" : "day"}
                  onClick={() => setDay(d.day)}
                >
                  {d.day.slice(5)}
                  {d.unread + d.viewed > 0 && <i />}
                </button>
              ))}
            </div>
            {today && (
              <div className="summary">
                <div className="tiles">
                  <div className="tile">
                    <span>Unread</span>
                    <strong>{today.unread}</strong>
                  </div>
                  <div className="tile">
                    <span>Viewed</span>
                    <strong className="count-viewed">{today.viewed}</strong>
                  </div>
                  <div className="tile">
                    <span>Finished</span>
                    <strong className="count-done">{today.finished}</strong>
                  </div>
                </div>
                <span className="progress">
                  <span style={{ width: `${(100 * today.finished) / (briefings.length || 1)}%` }} />
                </span>
              </div>
            )}
            {briefings.length === 0 ? (
              <p className="empty">No briefings yet. They arrive at 07:00.</p>
            ) : (
              <ul>
                {order.map((b) => (
                  <BriefingRow
                    key={b.id}
                    briefing={b}
                    onOpen={() => openBriefing(b)}
                    onSwipe={(action) => void swipe(b, action)}
                  />
                ))}
              </ul>
            )}
          </>
        )}
      </main>
      {undo && (
        <div className="toast">
          <span>{undo.text}</span>
          <button
            type="button"
            onClick={() => {
              void change(undo.id, undo.status);
              setUndo(null);
            }}
          >
            Undo
          </button>
        </div>
      )}
      {popup && (
        <div className="backdrop">
          <div className="dialog" role="dialog" aria-label="New briefings">
            <h2>Good morning</h2>
            <p>
              {left.length} new briefing{left.length === 1 ? "" : "s"} for {day}.
            </p>
            <ul>
              {left.map((b) => (
                <li key={b.id}>{b.topic}</li>
              ))}
            </ul>
            <div className="buttons">
              <button type="button" onClick={closePopup}>
                Later
              </button>
              <span className="grow" />
              <button
                type="button"
                className="primary"
                onClick={() => {
                  closePopup();
                  const first = left[0];
                  if (first) openBriefing(first);
                }}
              >
                Start reading
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
