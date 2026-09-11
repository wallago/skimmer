self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.skimmer;
  settingsFormat = pkgs.formats.toml { };
  configFile = settingsFormat.generate "config.toml" cfg.settings;
in
{
  options.services."skimmer" = {
    enable = lib.mkEnableOption "AI agent that reads your RSS feeds and keeps only what's worth your time.";

    dry_run = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Build the prompt and print it without calling the API.";
    };

    verbose = lib.mkOption {
      type = lib.types.ints.between 0 3;
      default = 1;
      example = 3;
      description = ''
        Verbosity level, passed to the app as repeated `-v` flags — `2` becomes `-vv`.

        - `0`: error / warn only, no `-v` flag
        - `1`: info
        - `2`: debug
        - `3`: traces
      '';
    };

    settings = lib.mkOption {
      inherit (settingsFormat) type;
      default = { };
      example = {
        output = "/home/wallago/sync-notes/skimmer/";
        miniflux = {
          url = "http://localhost";
          user = "admin";
          password = "passwoard";
          password_file = "./to/password/file";
        };
        claude = {
          model = "claude-haiku-4-5";
          api_key = "sk-key";
          api_key_file = "./to/key/file";
        };
        topic = {
          rust = {
            question = "What's going on interesting in the Rust world the past day?";
            feeds = [
              "Reddit Rust"
              "This Week in Rust"
            ];
            interval = "24h";
          };
        };
      };
      description = ''
        Configuration written to tool's config.toml.
      '';
    };

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "skimmer.packages.\${system}.default";
      description = "AI agent that reads your RSS feeds and keeps only what's worth your time.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    systemd.services.skimmer = {
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      serviceConfig =
        let
          verboseFlag = lib.optionalString (
            cfg.verbose > 0
          ) " -${lib.concatStrings (lib.replicate cfg.verbose "v")}";
        in
        {
          Type = "oneshot";
          ExecStart = "${lib.getExe cfg.package} --config %d/config${verboseFlag}${lib.optionalString cfg.dry_run " --dry-run"}";
          LoadCredential = "config:${configFile}";
          StateDirectory = "skimmer";
        };
    };
    systemd.timers.skimmer = {
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = "daily";
        Persistent = true;
        RandomizedDelaySec = "15m";
      };
    };

    environment.etc."skimmer/config.toml" = lib.mkIf (cfg.settings != { }) {
      source = configFile;
    };
  };
}
