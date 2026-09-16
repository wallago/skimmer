{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
    claude-code = {
      url = "github:sadjow/claude-code-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      naersk,
      claude-code,
      ...
    }:
    # ── System-agnostic outputs (modules) live out here ──
    {
      nixosModules.default = import ./nix/module.nix self;
      homeModules.default = import ./nix/hm-module.nix self;
    }
    # ── Then merge the per-system outputs onto it ──
    //
      flake-utils.lib.eachSystem
        [
          "x86_64-linux"
          "aarch64-linux"
        ]
        (
          system:
          let
            overlays = [ (import rust-overlay) ];
            pkgs = import nixpkgs {
              inherit system overlays;
              config.allowUnfree = true;
            };

            # ── Toolchain ─────────────────────────────────────────────
            rust = pkgs.rust-bin.nightly.latest.default.override {
              extensions = [
                "llvm-tools-preview"
                "rust-src"
              ];
            };

            naersk' = pkgs.callPackage naersk {
              cargo = rust;
              rustc = rust;
            };

            # ── Build helper ──────────────────────────────────────────
            buildApp =
              { release }:
              let
                name = "skimmer";
                desc = "AI agent that reads your RSS feeds and keeps only what's worth your time.";
              in
              pkgs.callPackage ./nix/package.nix {
                inherit
                  naersk'
                  release
                  name
                  desc
                  ;
                src = ./.;
              };

            # ── Claude Settings ─────────────────────────────────────
            claude = claude-code.packages.${system}.default;

            # ── Tooling shared by the dev shell and CI ───────────────
            ciTools = with pkgs; [
              rust
              # rust tooling
              cargo-nextest
              cargo-deny
              cargo-audit
              cargo-machete
              cargo-edit
              cargo-llvm-cov
              typos
              committed
              git-cliff
              taplo
              editorconfig-checker
              lychee

              # nix tooling
              nixfmt
              statix
              deadnix

              # crate deps
            ];
          in
          {
            # ── Packages ──────────────────────────────────────────────
            packages = rec {
              skimmer = buildApp { release = true; };
              skimmer-debug = buildApp { release = false; };
              default = skimmer;
            };

            # ── Checks (nix flake check) ─────────────────────────────
            checks.check = self.packages.${system}.skimmer-debug;

            # ── Dev Shell (nix develop) ──────────────────────────────
            devShells.default =
              let
                banner = pkgs.writeShellApplication {
                  name = "project-banner";
                  runtimeInputs = with pkgs; [
                    gum
                    jq
                    git
                    coreutils
                    findutils
                  ];
                  text = ''
                    export CLICOLOR_FORCE=1
                    cd "$(git rev-parse --show-toplevel)"

                    created=$(git log --reverse --format=%as | sed -n 1p)
                    updated=$(git log -1 --format=%cr)
                    commits=$(git rev-list --count HEAD)
                    nixpkgs=$(date -d @"$(jq -r .nodes.nixpkgs.locked.lastModified flake.lock)" +%F)

                    title=$(gum style --foreground 111 --bold 'skimmer')
                    desc=$(gum style --foreground 244 --italic "AI agent that reads your RSS feeds and keeps only what's worth your time.")
                    keys=$(gum style --foreground 80 --align right --padding "0 2 0 0" \
                      created updated commits nixpkgs)
                    vals=$(gum style --foreground 255 \
                      "$created" "$updated" "$commits" "$nixpkgs")

                    header=$(gum join --vertical --align center "$title" "" "$desc")

                    body=$(gum join --vertical --align center \
                      "$header" "" "$(gum join --horizontal "$keys" "$vals")")

                    gum style --border double --border-foreground 111 \
                      --margin "1 2" --padding "1 4" "$body"
                  '';
                };
              in
              pkgs.mkShell {
                PROJECT_BANNER = pkgs.lib.getExe banner;
                buildInputs =
                  ciTools
                  ++ (with pkgs; [
                    rust-analyzer
                    just
                    claude
                    nodejs
                  ]);
              };

            # ── CI Shell (nix develop .#ci) ──────────────────────────
            # Lean: just the toolchain + checks, no editor/claude/shellHook.
            devShells.ci = pkgs.mkShell {
              buildInputs = ciTools;
            };
          }
        );
}
