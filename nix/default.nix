{
  # TODO: This is silly, find out why pkgs is not being passed here
  # (or find a better workaround)
  mkMkCIConfig = { self, pkgs, ... }:
    config:
    let
      buildInfoPath = "${self.sourcePath}/build-info.json";
      event =
        if
          (builtins.pathExists buildInfoPath)
        then
          builtins.fromJSON (builtins.readFile buildInfoPath)
        else
        # TODO: fake build data?
          { };
    in
    (pkgs.lib.evalModules {
      modules = [
        # ./github-release.nix
        ./buildkite.nix
        ./find-derivations.nix
        config
      ];

      specialArgs = { inherit self pkgs event; };
    });
}
