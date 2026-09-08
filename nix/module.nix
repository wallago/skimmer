self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.repo-builder;
in
{
  options.programs."skimmer" = {
    enable = lib.mkEnableOption "AI agent that reads your RSS feeds and keeps only what's worth your time.";

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
      serviceConfig = {
        Type = "oneshot";
        ExecStart = "${cfg.package} --config %d/config";
        LoadCredential = "config:${cfg.configFile}";
        StateDirectory = "skimmer"; # /var/lib/skimmer
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
  };
}
