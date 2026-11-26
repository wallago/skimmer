import { useEffect, useRef } from "react";
import type { Briefing } from "../api";

/**
 * One briefing, full page. Reaching its end calls `onFinished` once, which
 * marks it finished.
 */
export function Reader({
  briefing,
  onBack,
  onFinished,
}: {
  briefing: Briefing;
  onBack: () => void;
  onFinished: () => void;
}) {
  const end = useRef<HTMLDivElement>(null);
  const finished = briefing.status === "finished";

  useEffect(() => {
    const node = end.current;
    if (finished || !node || typeof IntersectionObserver === "undefined") return;
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) {
        observer.disconnect();
        onFinished();
      }
    });
    observer.observe(node);
    return () => observer.disconnect();
  }, [finished, onFinished]);

  const source = (id: number) => briefing.sources.find((s) => s.id === id);

  return (
    <article className="reader">
      <button type="button" className="back" onClick={onBack}>
        ‹ Back
      </button>
      <p className="topic">
        {briefing.topic} · {briefing.day}
      </p>
      <h1>{briefing.question}</h1>
      {briefing.digest.split("\n\n").map((paragraph) => (
        <p className="digest" key={paragraph}>
          {paragraph}
        </p>
      ))}
      <h2>Highlights</h2>
      <ul>
        {briefing.highlights.map((h) => (
          <li className="hl" key={h.headline}>
            <b>{h.headline}</b>
            <p>{h.detail}</p>
            <span className="meta">
              {h.entry_ids.map((id) => {
                const s = source(id);
                return s ? (
                  <a className="tag" key={id} href={s.url} target="_blank" rel="noreferrer">
                    {s.feed}
                  </a>
                ) : null;
              })}
            </span>
          </li>
        ))}
      </ul>
      <details>
        <summary>Sources: {briefing.scanned} entries</summary>
        <ul>
          {briefing.sources.map((s) => (
            <li key={s.id}>
              <a href={s.url} target="_blank" rel="noreferrer">
                {s.title}
              </a>{" "}
              <span className="muted">{s.feed}</span>
            </li>
          ))}
        </ul>
      </details>
      <div ref={end} className={finished ? "end done" : "end"}>
        {finished ? "✓ Finished" : "Scroll to the end to finish"}
      </div>
    </article>
  );
}
