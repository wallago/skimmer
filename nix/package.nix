{
  lib,
  naersk',
  src,
  release ? true,
  name,
  desc,
}:
naersk'.buildPackage {
  inherit name src release;
  # The root Cargo.toml is a bare workspace, so naersk finds no version there
  # and traces its whole config; read it from the package instead.
  inherit ((lib.importTOML (src + "/apps/server/Cargo.toml")).package) version;

  meta = {
    description = desc;
    mainProgram = name;
    license = with lib.licenses; [
      asl20
      mit
    ];
  };
}
