{ config, pkgs, lib, event, ... }:

let
  inherit (lib) types;
in
{
  options = {
    nixpkgs-fmt = {
      package = lib.mkOption {
        type = types.package;
        default = pkgs.nixpkgs-fmt;
        description = ''
          The derivation containing the `nixpkgs-fmt` tool.
        '';
      };

      checks = lib.mkOption {
        description = ''
          The set of `nixpkgs-fmt` checks to run.
        '';

        default = { };

        type = types.attrsOf (types.submodule {
          options = {
            enable = lib.mkOption {
              type = types.bool;
              default = true;
              description = ''
                If true (the default) this nixpkgs-fmt check will be run.
              '';
            };

            srcs = lib.mkOption {
              type = types.listOf types.path;
              description = ''
                Path/s to files/directories to check.
              '';
            };

            # TODO: put these in a lib?
            dependsOn = lib.mkOption {
              type = types.listOf types.str;
              default = [ "wait-builds" ];
              description = ''
                Arbitrary list of steps to wait on. If not provided, waits until
                all builds have been completed.
              '';
            };

            timeoutMinutes = lib.mkOption {
              type = types.int;
              default = 5;
              description = ''
                Number of minutes to wait for release to be completed.
              '';
            };
          };
        });
      };
    };
  };

  config =
    let
      name' = key: "nixpkgs-fmt-${key}";
      buildFmtStep = key: cfg: {
        name = name' key;
        value = {
          config = {
            type = "command";
            label = ":nix: :broom: ${key}";
            depends_on = cfg.dependsOn;
            timeout_in_minutes = cfg.timeoutMinutes;
          };
        };
      };

      buildFmtCommand = key: cfg:
        let
          srcs = lib.concatStrings (lib.intersperse " " cfg.srcs);
        in
        {
          name = name' key;
          value = "${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt --check ${srcs}";
        };
    in
    {
      steps = lib.mapAttrs' buildFmtStep config.nixpkgs-fmt.checks;
      commands = lib.mapAttrs' buildFmtCommand config.nixpkgs-fmt.checks;
    };
}
