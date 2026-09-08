# Claude API Advice

1. Claude follows tagged structure much more reliably than a flat blob.

## Building the Prompt

2. Instructions go in `system`, data goes in the user message. Don't mix them.
   The split is also what makes the untrusted-content boundary meaningful:
   everything in `system` is yours, everything in `messages` may be hostile.

3. State the output contract in `system`, not in every user message. It's the
   part that never changes between topics.

4. Order of assembly is `tools` -> `system` -> `messages`. Anything volatile
   (timestamps, the topic question) belongs after the stable part.

5. Models fabricate plausible URLs. Never ask for a link back — give entries an
   id, ask for the id, resolve it to a URL yourself.

## Request Parameters

6. Model id is `claude-opus-5`, complete as-is. Never append a date suffix —
   `claude-opus-5-20260101` is not a real model.

7. `max_tokens` 16000 is the right default for non-streaming (yours is correct).
   Anything much larger needs streaming or you'll hit HTTP timeouts. It's a hard
   cutoff, not a target — hitting it truncates mid-sentence.

8. Thinking is ON by default on Opus 5. Omitting `thinking` runs adaptive.

9. `budget_tokens` is dead on Opus 5 — sending it is a 400, not a warning.
   Depth is controlled by `output_config: {effort: "low"|...|"max"}`, default
   `high`. `effort` is nested inside `output_config`, not top-level.

10. Assistant prefill is removed on Opus 5 — a trailing assistant message to
    force "# Report" as the first token returns 400. Use structured outputs.

11. Structured outputs (`output_config.format`) beat parsing markdown back out
    of prose. Deserialize straight into `Briefing` / `Highlight` / `Source`.

## Response Handling

12. Always check `stop_reason` before reading `content`.

13. `pause_turn` is real and you WILL hit it with web_search. The server-side
    tool loop caps at 10 iterations; you re-send the user message plus the
    assistant response and it resumes. Do NOT append "Continue." — the API
    detects the trailing server_tool_use block. Cap your retries (~5).

14. Server-tool errors do NOT raise. A failed web search is HTTP 200 with a
    `web_search_tool_result` block whose `content` is an error OBJECT
    ({error_code: "max_uses_exceeded"}) instead of the usual LIST. Branch on
    the shape before you index, or serde will fail on a successful request.

15. `stop_details` is an object ({type, category, explanation}), populated only
    when stop_reason == "refusal", null otherwise. Not a String.

16. Set `max_uses` on the web_search tool. Without it a single topic can burn a
    lot of searches.

## Cost

17. POST /v1/messages/count_tokens gives exact counts for a prompt without
    running it. Same request body, no charge. Use it to see what an entry batch
    really costs before sending. Never estimate with tiktoken — it's OpenAI's
    tokenizer and undercounts Claude by 15-20%.

18. Prompt caching will NOT help this app. The cache TTL is 5 minutes (1h opt-in)
    and runs are 24h apart, so every run is a cold cache. Don't build for it.

19. The Batch API is 50% cost and fits this workload exactly: N independent
    topics, nightly, nobody waiting on the response. Submit all topics as one
    batch, poll until processing_status == "ended". Results come back in ANY
    order — key by custom_id, never by position.

20. reqwest does not retry. 429 and 5xx need your own backoff, or one blip kills
    the whole nightly run.
