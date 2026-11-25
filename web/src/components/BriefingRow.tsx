import { type PointerEvent, useRef, useState } from "react";
import type { Briefing } from "../api";
import { type SwipeAction, swipeAction } from "../swipe";

/** One briefing in the day's list: tap to open, swipe right to read, left for later. */
export function BriefingRow({
  briefing,
  onOpen,
  onSwipe,
}: {
  briefing: Briefing;
  onOpen: () => void;
  onSwipe: (action: SwipeAction) => void;
}) {
  const start = useRef<{ x: number; y: number } | null>(null);
  const [dx, setDx] = useState(0);
  const first = briefing.highlights[0];

  const down = (event: PointerEvent) => {
    start.current = { x: event.clientX, y: event.clientY };
  };
  const move = (event: PointerEvent) => {
    if (start.current) setDx(event.clientX - start.current.x);
  };
  const up = (event: PointerEvent) => {
    const from = start.current;
    start.current = null;
    setDx(0);
    if (!from) return;
    const moveX = event.clientX - from.x;
    const action = swipeAction(moveX, event.clientY - from.y);
    if (action) onSwipe(action);
    else if (Math.abs(moveX) < 6) onOpen();
  };

  return (
    <li className="row">
      <div className={`under ${dx < 0 ? "left" : "right"}`}>{dx < 0 ? "Later ⏱" : "✓ Read"}</div>
      <button
        type="button"
        className={`card ${briefing.status}`}
        style={{ transform: dx ? `translateX(${dx}px)` : undefined }}
        onPointerDown={down}
        onPointerMove={move}
        onPointerUp={up}
        onPointerCancel={() => {
          start.current = null;
          setDx(0);
        }}
        onClick={(event) => {
          // Keyboard Enter or Space: pointer taps are handled on pointer up.
          if (event.detail === 0) onOpen();
        }}
      >
        <span className="dot" />
        <span className="body">
          <span className="topic">{briefing.topic}</span>
          <span className="head">{first?.headline ?? briefing.question}</span>
          <span className="prev">{briefing.digest}</span>
        </span>
      </button>
    </li>
  );
}
