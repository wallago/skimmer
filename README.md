# skimmer

AI agent that reads your RSS feeds and keeps only what's worth your time.

`skimmer` pulls the entries your [Miniflux](https://miniflux.app) server collected since the
last run, groups them into the topics you define, asks Claude one question per topic, and
writes a single markdown report.

## How it works

1. **Load** — config from `--config`, else `$XDG_CONFIG_HOME/skimmer/config.toml`; state from
   `$XDG_STATE_HOME/skimmer/state.toml`.
2. **Resolve topics** — each `[topic.<name>]` matches the server's feeds whose title *contains*
   one of its `feeds` strings.
3. **Window** — entries published after `--since`, else after that topic's last run, else one
   `interval` ago.
4. **Ask** — entries are rendered as escaped `<entry id="…">` blocks and sent to the Messages
   API with a JSON schema, so the answer comes back as a digest plus cited highlights.
5. **Write** — `report_<timestamp>.md` in `output`, one section per topic, with source links
   back to each entry.
6. **Record** — each briefed topic's `last_run` is stamped, so the next run picks up where this
   one stopped.

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

Copy [`config.template.toml`](config.template.toml) to `$XDG_CONFIG_HOME/skimmer/config.toml`
and fill it in. Unknown top-level keys are rejected.

```toml
output = "/home/you/notes/skimmer"   # existing directory the reports are written to

[miniflux]
url      = "https://rss.example.com"
username = "admin"
password = "…"                        # or password_file = "/run/secrets/miniflux"

[claude]
api_key = "sk-ant-…"                  # or api_key_file = "/run/secrets/anthropic"
model   = "claude-opus-5"

[topic.rust]
question = "What's going on interesting in the Rust world the past day?"
feeds    = ["Reddit Rust", "This Week in Rust"]
interval = "7d"
```

| Key | Meaning |
| --- | --- |
| `output` | Directory the reports land in. Must already exist. |
| `miniflux.url` / `.username` | Server and the account to read feeds as. |
| `miniflux.password` / `.password_file` | One of the two is required; the file wins and its trailing newline is trimmed. |
| `claude.api_key` / `.api_key_file` | Same rule — one of the two is required. |
| `claude.model` | Model id, e.g. `claude-haiku-4-5`, `claude-sonnet-5`, `claude-opus-5`. |
| `topic.<name>.question` | What Claude is asked about this topic's entries. |
| `topic.<name>.feeds` | Feed-title substrings; a feed matches if its title contains one. |
| `topic.<name>.interval` | How far back to look on a first run, e.g. `"24h"`, `"7d"`. |

Secret files are the reason `password_file` / `api_key_file` exist: point them at a sops-nix or
systemd credential path and keep the key out of the config.

Two caveats on the template: `output` is not in it but is required in practice, and the
`categories` key on `[topic.bicycle]` is not implemented — only `feeds` selects feeds today.

## Usage

```
skimmer [OPTIONS]

  -v, --verbose...     Increase logging verbosity (-v, -vv, -vvv)
      --config <PATH>  Path to the application config file (TOML)
  -s, --since <SINCE>  Start the window at this timestamp instead of the last run
      --dry-run        Count tokens and log the estimated cost without calling the model
  -h, --help           Print help
  -V, --version        Print version
```

`--dry-run` prices the prompt against a built-in rate table (haiku 4.5, sonnet 5, opus 5) and
leaves both the report and the state file untouched. A `--feeds` flag is also accepted but is
currently a no-op.

Logs go to stderr — plain under journald, timestamped otherwise. Default level is warnings and
errors; `-v` adds the per-topic progress.

## Running it on a schedule

The flake ships a NixOS module (`nixosModules.default`) that installs the binary, renders the
config through `LoadCredential`, and runs it from a daily persistent timer:

```nix
{
  inputs.skimmer.url = "github:wallago/skimmer";

  # …

  imports = [ inputs.skimmer.nixosModules.default ];

  services.skimmer = {
    enable  = true;
    verbose = 1;
    settings = {
      output = "/var/lib/skimmer";
      miniflux = {
        url           = "https://rss.example.com";
        username      = "admin";
        password_file = "/run/secrets/miniflux";
      };
      claude = {
        model        = "claude-haiku-4-5";
        api_key_file = "/run/secrets/anthropic";
      };
      topic.rust = {
        question = "What's going on interesting in the Rust world the past day?";
        feeds    = [ "Reddit Rust" "This Week in Rust" ];
        interval = "24h";
      };
    };
  };
}
```

`homeModules.default` is the home-manager counterpart; it installs the package only.

## Development

`nix develop` brings in the toolchain and every tool below; [`just`](justfile) drives them:

```sh
just check         # cargo check --all-targets
just test          # cargo nextest run
just lint          # clippy, warnings denied
just fmt           # rustfmt + nixfmt
just coverage      # cargo llvm-cov nextest
just audit / deny  # RustSec advisories, license & source policy
just flake-check   # every flake output, all systems
```

Tests are unit tests next to the code, with [wiremock](https://docs.rs/wiremock) standing in for
Miniflux and fixtures under `tests/fixtures/`. Commits follow Conventional Commits (`just
commits` checks them) and the changelog is generated with git-cliff.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
