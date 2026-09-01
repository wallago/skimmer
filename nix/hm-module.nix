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
    home.packages = [ cfg.package ];
  };
}
