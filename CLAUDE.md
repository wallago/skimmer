# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:

- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

## Apps

skimmer reads the entries a Miniflux server collected, asks Claude for one
briefing per topic per day, saves them in SQLite, and serves them to a phone
PWA. Each app has its own folder, its own `justfile` and its own checks. The
root `justfile` loads each one as a module, and `just ci` runs every app's
`ci` after the repo-wide checks.

- `apps/server/`: the `skimmer` binary (daily run and `skimmer serve`); its
  recipes run as `server::<recipe>`.
- `apps/web/`: Vite + React + TypeScript PWA, pnpm; its recipes run as
  `web::<recipe>`. `skimmer serve` serves its `dist/` at `/` from `[web] dir`.

## Layout

- `flake.nix`: dev shell (`nix develop`) and the lean CI shell (`nix develop .#ci`). Tools the checks run go in `ciTools`, editor-only ones in `devTools`. The package and the NixOS / home-manager modules come from `nix/`.
- `apps/server/src/main.rs`, `lib.rs`: parse `Args`, then either the daily run (`app/`) or `serve` (`server/`).
- `apps/server/src/app/`: one briefing per topic: fetch from Miniflux (`ext/rss.rs`), ask Claude (`ext/claude/`), save the briefing and the topic's last run in the store (`store.rs`, SQLite with `PRAGMA user_version` migrations). Nothing else is written to disk.
- `apps/server/src/server/`: axum routes under `/api` (`briefings.rs`) and the built web app (`web.rs`).
- `apps/server/src/config/`: `Config`, read from TOML; one table per component, each listed in `config.example.toml`.
- `apps/server/src/error.rs`: the crate's one `Error` enum.
- `apps/server/tests/fixtures/`: files the unit tests load.
- `apps/web/src/`: the PWA; `api.ts` talks to `/api`, `App.tsx` is the page, `badge.ts` puts the unread count on the icon and shows the notifications, `sw.ts` is the service worker.
- `.github/workflows/ci.yml`: one job per `just ci` recipe, each run in the flake's CI shell, plus coverage, the Nix flake check, the commit-format check, the cross build and MSRV.

## Rules

Standalone rules live in `.claude/rules/` and are imported here. Only files
reachable from `CLAUDE.md` get loaded, so a new rule needs a line below.

@.claude/rules/nix.md

## Commands

`just` is the entry point, not the raw toolchain; `just --list` shows every
recipe.

- `just ci`: the gate. It runs every check CI runs, recipe by recipe, so a
  green `just ci` is a green CI. Run it before calling a change done.
- `just --list <app>` lists an app's recipes; run them as `<app>::check`,
  `<app>::test`, `<app>::lint`, `<app>::fmt` for the fast loop.

## Constraints

- Clippy runs with `-D warnings` and `Cargo.toml` warns on `unwrap_used`,
  `expect_used`, `panic`, `todo`, `print_stdout` and
  `missing_docs_in_private_items`: no `.unwrap()`, no `.expect()`, no bare
  `panic!`, and every item gets a doc comment.
- `cargo machete` fails on a dependency nothing uses; drop it in the change that orphans it.
- A new crate has to clear `cargo deny check`: licences, bans, sources and advisories.
- `cargo audit` fails on a dependency with a RustSec advisory.
- TypeScript is `strict` with `noUncheckedIndexedAccess`: no `any`, no `!`
  non-null assertions; narrow instead.
- Biome lints with warnings as errors and owns formatting and import order:
  let `biome check --write` format, don't hand-format.
- Tests use Testing Library: query by role or text, as a user would, not by
  class name or test id.
- Dependencies are locked in `pnpm-lock.yaml`; add one with `pnpm add`, and say
  what it buys before pulling it in.
- The service worker only runs from a production build: check offline
  behaviour with `vite build` then `vite preview`, never the dev server.
- `editorconfig-checker` holds every file to `.editorconfig`: indentation, final newline, no trailing spaces.
- `typos` runs over the repo. Add a real term to `typos.toml` rather than rewording around it.
- Every link in Markdown is checked. A placeholder URL fails the gate.
- Commit messages are Conventional Commits (`type(scope): subject`); CI checks every commit of a PR with `committed`, and `git-cliff` builds the changelog from them.

## Verifying a change

`just ci` green is the bar. Say which recipes ran when reporting a change.

The gate can't check the phone UI or the live services: for the PWA, run
`skimmer serve` on a seeded database and look at it at phone width. Miniflux
is stood in for by wiremock in the tests; a real Claude call needs
`--dry-run` off and an API key.
