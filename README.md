# skimmer

AI agent that reads your RSS feeds and keeps only what's worth your time.

`skimmer` pulls the entries your [Miniflux](https://miniflux.app) server collected since the
last run, groups them into the topics you define, asks Claude one question per topic, and
saves each answer to a SQLite database that `skimmer serve` shows on your phone.

## How it works

1. **Load** — config from `--config`, else `$XDG_CONFIG_HOME/skimmer/config.toml`; each topic's
   last run from the `database`.
2. **Resolve topics** — each `[topic.<name>]` matches the server's feeds whose title *contains*
   one of its `feeds` strings.
3. **Window** — entries published after `--since`, else after that topic's last run, else one
   `interval` ago.
4. **Ask** — entries are rendered as escaped `<entry id="…">` blocks and sent to the Messages
   API with a JSON schema, so the answer comes back as a digest plus cited highlights.
5. **Save** — each briefing goes to the `database`, unread, with its source links back to each
   entry.
6. **Record** — each briefed topic's `last_run` is stamped in the `database`, so the next run
   picks up where this one stopped.

Feed content is untrusted: XML metacharacters are escaped before it reaches the prompt, and the
system prompt tells Claude to summarize entry content, never to follow instructions inside it.

## Install

With Nix:

```sh
nix run github:wallago/skimmer
nix build github:wallago/skimmer   # ./result/bin/skimmer
```

From source:

```sh
cargo build --release   # target/release/skimmer
```

## Configuration

Copy [`config.example.toml`](apps/server/config.example.toml) to `$XDG_CONFIG_HOME/skimmer/config.toml`
and fill it in. Unknown top-level keys are rejected.

```toml
[miniflux]
url      = "https://rss.example.com"
username = "admin"
password = "…"                        # or password_file = "/run/secrets/miniflux"

[claude]
api_key = "sk-ant-…"                  # or api_key_file = "/run/secrets/anthropic"
model   = "claude-haiku-5-5"

[topic.rust]
question = "What's going on interesting in the Rust world the past day?"
feeds    = ["Reddit Rust", "This Week in Rust"]
interval = "7d"
```

| Key | Meaning |
| --- | --- |
| `database` | Optional. SQLite file the briefings and each topic's last run are saved to. Defaults to `$XDG_STATE_HOME/skimmer/skimmer.db`. |
| `server.address` / `.port` | Where `skimmer serve` listens. Defaults to `127.0.0.1` and `3000`. |
| `web.dir` | Optional. Built web app (`apps/web/dist`) that `skimmer serve` serves at `/`. |
| `miniflux.url` / `.username` | Server and the account to read feeds as. |
| `miniflux.password` / `.password_file` | One of the two is required; the file wins and its trailing newline is trimmed. |
| `claude.api_key` / `.api_key_file` | Same rule — one of the two is required. |
| `claude.model` | Model id, e.g. `claude-haiku-5-5`, `claude-sonnet-5-5`, `claude-opus-5-5`. |
| `topic.<name>.question` | What Claude is asked about this topic's entries. |
| `topic.<name>.feeds` | Feed-title substrings; a feed matches if its title contains one. |
| `topic.<name>.interval` | How far back to look on a first run, e.g. `"24h"`, `"7d"`. |

Secret files are the reason `password_file` / `api_key_file` exist: point them at a sops-nix or
systemd credential path and keep the key out of the config.

One caveat on the template: the `categories` key on `[topic.bicycle]` is not implemented — only
`feeds` selects feeds today.

## Usage

```
skimmer [OPTIONS] [COMMAND]

  serve                Serve the saved briefings over HTTP until stopped

  -v, --verbose...     Increase logging verbosity (-v, -vv, -vvv)
      --config <PATH>  Path to the application config file (TOML)
  -s, --since <SINCE>  Start the window at this timestamp instead of the last run
      --dry-run        Count tokens and log the estimated cost without calling the model
  -h, --help           Print help
  -V, --version        Print version
```

`--dry-run` prices the prompt against a built-in rate table (haiku 4.5, sonnet 5, opus 5) and
leaves the database untouched. A `--feeds` flag is also accepted but is
currently a no-op.

### `skimmer serve`

Serves the saved briefings from `database` for the web app, until Ctrl-C or SIGTERM:

| Route | Answer |
| --- | --- |
| `GET /health` | `200` while the server is up |
| `GET /api/days` | Every day with briefings, newest first: `[{day, unread, viewed, finished, input_tokens, output_tokens, cost_usd}]`. `cost_usd` is priced when each briefing is written, and is `null` for a model missing from the price table |
| `GET /api/days/{day}` | That day's briefings (`day` like `2026-11-23`), with digest, highlights, sources and `status` |
| `PUT /api/briefings/{id}/status` | Body `{"status": "unread" \| "viewed" \| "finished"}`; answers the briefing, `404` if there is none |

There is no login: keep it on `127.0.0.1`, or reach it over a VPN or a reverse proxy.

With `[web] dir` set to a built web app (`apps/web/dist`), `serve` also serves it at `/`.

### The phone app (`apps/web/`)

A React PWA scaffolded from praline's `web.react` and `web.pwa` templates, in wallago/todo's
colours. Install it from the browser ("Add to Home Screen") and it opens offline.

- **Today:** a strip of days, unread / viewed / finished tiles, and one row per topic. Tap a row
  to read it (marks it viewed); scroll to its end to finish it. Swipe right to mark it read, left
  to keep it for later, with an undo.
- **Morning popup:** the first time the app opens on a day with new briefings, or when they land
  while it is open. The app checks every 30 s while on screen, and on coming back to it.
- **Icon badge:** the number of briefings not finished yet, where the browser supports it.
- **Notifications:** after a tap on "Show new briefings as notifications", one quiet notification
  per unread briefing of the newest day, closed once it is opened, as wallago/todo does for due
  todos. On Android this is what badges the icon. They come from the app itself, not a push
  server, so they appear when the app runs, not at 07:00 sharp with the app closed.
- **Usage:** tokens and cost per day, with the total.

```sh
just web::dev      # Vite on :5173, /api proxied to a local `skimmer serve` on :3000
just web::build    # apps/web/dist, for `[web] dir`
just web::test     # vitest; `web::check`, `web::lint`, `web::fmt-check` too
```

Logs go to stderr — plain under journald, timestamped otherwise. Default level is warnings and
errors; `-v` adds the per-topic progress.

## Running it on a schedule

The flake ships a NixOS module (`nixosModules.default`) that installs the binary, renders the
config through `LoadCredential`, and runs it from a persistent timer, every day at 07:00
Europe/Paris unless `schedule` says otherwise. `serve = true` also runs `skimmer serve` as a
service. Both units share `/var/lib/skimmer/skimmer.db`:

```nix
{
  inputs.skimmer.url = "github:wallago/skimmer";

  # …

  imports = [ inputs.skimmer.nixosModules.default ];

  services.skimmer = {
    enable   = true;
    verbose  = 1;
    schedule = "*-*-* 07:00:00 Europe/Paris"; # the default
    serve    = true;
    firewallInterfaces = [ "wg0" ]; # open the serve port on these interfaces only
    settings = {
      miniflux = {
        url           = "https://rss.example.com";
        username      = "admin";
        password_file = "/run/secrets/miniflux";
      };
      claude = {
        model        = "claude-haiku-5-5";
        api_key_file = "/run/secrets/anthropic";
      };
      topic.rust = {
        question = "What's going on interesting in the Rust world the past day?";
        feeds    = [ "Reddit Rust" "This Week in Rust" ];
        interval = "24h";
      };
      web.dir = "/srv/skimmer-web"; # a copy of apps/web/dist, from `just web::build`
    };
  };
}
```

`homeModules.default` is the home-manager counterpart; it installs the package only.

## Development

`nix develop` brings in the toolchain and every tool below; [`just`](justfile) drives them:

```sh
just ci                # every check CI runs; green here is green there
just server::check     # cargo check --all-targets
just server::test      # cargo nextest run
just server::lint      # clippy, warnings denied
just server::fmt       # rustfmt + taplo; `just fmt-nix` for the Nix sources
just coverage          # cargo llvm-cov, into lcov.info
just audit / deny      # RustSec advisories, license & source policy
just flake-check       # every flake output, all systems
```

Tests are unit tests next to the code, with [wiremock](https://docs.rs/wiremock) standing in for
Miniflux and fixtures under `apps/server/tests/fixtures/`. Commits follow Conventional Commits (`just
commits` checks them) and the changelog is generated with git-cliff.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
