{
  mkCIConfig = { self, pkgs, config ? { }, ... }:
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
        ./buildkite.nix
        ./find-derivations.nix
        ./lib/clippy.nix
        ./lib/rustfmt.nix
        config
      ];

      specialArgs = { inherit self pkgs event; };
    });
  # TODO: Generic modules will be re-exported here (e.g., releasing to github,
  # pushing containers, etc)
}
