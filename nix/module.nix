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
  verboseFlag = lib.optionalString (
    cfg.verbose > 0
  ) " -${lib.concatStrings (lib.replicate cfg.verbose "v")}";
  command = "${lib.getExe cfg.package} --config %d/config${verboseFlag}";
  # Both units read the same config and, through XDG_STATE_HOME, open the same
  # database: /var/lib/skimmer/skimmer.db.
  shared = {
    LoadCredential = "config:${configFile}";
    StateDirectory = "skimmer";
    Environment = "XDG_STATE_HOME=%S";
  };
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

    schedule = lib.mkOption {
      type = lib.types.str;
      default = "*-*-* 07:00:00 Europe/Paris";
      example = "*-*-* 06:30:00 UTC";
      description = ''
        When the daily briefings are written, as a systemd `OnCalendar` value.
        The time zone is part of the value, so it holds whatever the host's own.
      '';
    };

    firewallInterfaces = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      example = [ "wg0" ];
      description = "Open the `serve` port (`settings.server.port`, 3000 by default) on these interfaces only.";
    };

    serve = lib.mkEnableOption "the `skimmer serve` web app and API, at `settings.server` (127.0.0.1:3000 by default)";

    settings = lib.mkOption {
      inherit (settingsFormat) type;
      default = { };
      example = {
        output = "/home/wallago/sync-notes/skimmer/";
        reader = "User description.";
        miniflux = {
          url = "http://localhost";
          user = "admin";
          password = "passwoard";
          password_file = "./to/password/file";
        };
        claude = {
          model = "claude-haiku-5-5";
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
            context = "What the user want to know";
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

    systemd = {
      services.skimmer = {
        after = [ "network-online.target" ];
        wants = [ "network-online.target" ];
        serviceConfig = {
          Type = "oneshot";
          ExecStart = "${command}${lib.optionalString cfg.dry_run " --dry-run"}";
        }
        // shared;
      };
      timers.skimmer = {
        wantedBy = [ "timers.target" ];
        timerConfig = {
          OnCalendar = cfg.schedule;
          Persistent = true;
        };
      };

      services.skimmer-serve = lib.mkIf cfg.serve {
        description = "skimmer web app and API";
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" ];
        serviceConfig = {
          ExecStart = "${command} serve";
          Restart = "on-failure";
        }
        // shared;
      };
    };

    networking.firewall.interfaces = lib.mkIf cfg.serve (
      lib.genAttrs cfg.firewallInterfaces (_: {
        allowedTCPPorts = [ (cfg.settings.server.port or 3000) ];
      })
    );

    environment.etc."skimmer/config.toml" = lib.mkIf (cfg.settings != { }) {
      source = configFile;
    };
  };
}
