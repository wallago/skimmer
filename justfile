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
mod server 'apps/server'
mod web 'apps/web'

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
    @just server::ci
    @just web::ci

# Open an HTML report from the last coverage run (run `just coverage` first)
[group('dev')]
coverage-report:
    cargo llvm-cov report --html --open

# Format the Nix sources in place
[group('dev')]
fmt-nix:
    nixfmt flake.nix nix/package.nix nix/module.nix nix/hm-module.nix

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
