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

  meta = {
    description = desc;
    license = with lib.licenses; [
      asl20
      mit
    ];
  };
}
