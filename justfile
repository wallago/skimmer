import 'just/nix.just'
import 'just/udeps.just'
import 'just/deny.just'
import 'just/audit.just'
import 'just/codecov.just'
import 'just/editorconfig.just'
import 'just/typos.just'
import 'just/lychee.just'
import 'just/git.just'
import 'just/github.just'
import 'just/server.just'
mod web 'web'

# List all commands
default:
    @just --list

# Run every check before pushing
[group('ci')]
ci:
    @just nix-lint
    @just udeps
    @just deny
    @just audit
    @just editorconfig
    @just typos
    @just links
    @just workflows
    @just rust-ci
    @just web::ci

# The Rust checks: formatting, lints, tests and docs
[group('ci')]
rust-ci:
    @just fmt-check
    @just toml-check
    @just lint
    @just test
    @just doc

# Lint with clippy, treating warnings as errors
[group('quality')]
lint:
    cargo clippy --all-targets -- -D warnings

# Check Rust formatting
[group('quality')]
fmt-check:
    cargo fmt --check

# Check TOML formatting
[group('quality')]
toml-check:
    taplo fmt --check --diff

# Build the docs, treating warnings as errors
[group('quality')]
doc:
    RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps

# Open an HTML report from the last coverage run (run `just coverage` first)
[group('dev')]
coverage-report:
    cargo llvm-cov report --html --open

# Run the app. Usage: just run -- statement.csv bagels.db
[group('dev')]
run *ARGS:
    cargo run -- -v --config config.dev.toml {{ ARGS }}

# Build the optimized release binary
[group('dev')]
build:
    cargo build --release

# Type-check the whole workspace (fast feedback, no binary)
[group('dev')]
check:
    cargo check --all-targets

# Format Rust + Nix sources in place
[group('dev')]
fmt:
    cargo fmt  --all -- --check
    nixfmt flake.nix nix/package.nix nix/module.nix

# Run the test suite. Usage: just test [pattern]
[group('dev')]
test *ARGS:
    cargo nextest run {{ ARGS }}

# Format TOML files with taplo
[group('quality')]
fmt-toml:
    taplo fmt

# Cut & publish a release: bump version, changelog, commit, tag, push. Usage: just publish 0.2.0
[confirm("This will tag and push a release to origin. Continue?")]
[group('release')]
jj-publish VERSION:
    @test -z "$(jj diff --name-only)" || (echo "✗ working copy has uncommitted changes — commit or abandon them first" && exit 1)
    @echo "▶ bump version → {{ VERSION }}"
    cargo set-version {{ VERSION }}
    @echo "▶ regenerate changelog"
    @if command -v git-cliff >/dev/null 2>&1; then \
        git-cliff --tag v{{ VERSION }} -o CHANGELOG.md; \
    else \
        echo "⊘ git-cliff not installed — skipping changelog"; \
    fi
    @echo "▶ record release commit"
    jj commit -m "chore(release): v{{ VERSION }}"
    @echo "▶ advance main bookmark"
    jj bookmark set main -r @-
    @echo "▶ tag release commit"
    git tag -a v{{ VERSION }} -m "v{{ VERSION }}" "$(jj log -r @- --no-graph -T commit_id)"
    @echo "▶ push bookmark + tag"
    jj git push --bookmark main
    git push origin v{{ VERSION }}
    @echo "✅ published v{{ VERSION }}"
