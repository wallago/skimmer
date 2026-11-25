import { type Day, formatCost, formatTokens } from "../api";

/** Tokens and cost per day, newest first, with the total on top. */
export function Usage({ days }: { days: readonly Day[] }) {
  const priced = days.filter((d) => d.cost_usd !== null);
  const total = priced.length ? priced.reduce((sum, d) => sum + (d.cost_usd ?? 0), 0) : null;
  const tokens = days.reduce((sum, d) => sum + d.input_tokens + d.output_tokens, 0);

  return (
    <section>
      <div className="summary tiles2">
        <div className="tile">
          <span>Total cost</span>
          <strong>{formatCost(total)}</strong>
        </div>
        <div className="tile">
          <span>Tokens</span>
          <strong>{formatTokens(tokens)}</strong>
        </div>
      </div>
      <table className="usage">
        <thead>
          <tr>
            <th>Day</th>
            <th>In</th>
            <th>Out</th>
            <th>Cost</th>
          </tr>
        </thead>
        <tbody>
          {days.map((d) => (
            <tr key={d.day}>
              <td>{d.day}</td>
              <td>{formatTokens(d.input_tokens)}</td>
              <td>{formatTokens(d.output_tokens)}</td>
              <td>{formatCost(d.cost_usd)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
