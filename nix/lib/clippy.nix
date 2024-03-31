{ config, pkgs, lib, event, ... }:

let
  inherit (lib) types;
in
{
  options.clippy = {
    package = lib.mkOption {
      type = types.package;
      default = pkgs.clippy;
      description = ''
        The derivation containing the `clippy` tool.

        You may want to change this if you need to use a specific verison,
        or wrap it for some weird reason that I'm sure to find myself facing at
        some point.
      '';
    };

    checks = lib.mkOption {
      description = ''
        The set of `clippy` checks to run.
      '';

      default = { };

      type = types.attrsOf (types.submodule {
        options = {
          enable = lib.mkOption {
            type = types.bool;
            default = true;
            description = ''
              If true (the default) this `clippy` check will be run.
            '';
          };

          src = lib.mkOption {
            type = types.path;
            description = ''
              Directory with Cargo.lock
            '';
          };

          subdir = lib.mkOption {
            type = types.nullOr types.str;
            default = null;
            description = ''
              If specifed, will cd to this directory relative to `src` before
              executing `clippy`.
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

  config =
    let
      name' = key: "clippy-${key}";
      buildClippyStep = key: cfg: {
        name = name' key;
        value = {
          config = {
            type = "command";
            label = ":rust: :clippy: ${key}";
            depends_on = cfg.dependsOn;
            timeout_in_minutes = cfg.timeoutMinutes;
          };
        };
      };

      join = mid: elems:
        (lib.strings.concatStrings (lib.strings.intersperse " && " elems));

      buildClippyCmdScript = { package, subdir, src }:
        pkgs.writeShellScriptBin "run-clippy.sh" ''
          cd "''${BUILDKITE_REPO_LOCATION:-}"
          ${lib.optionalString (subdir != null) "cd ${subdir}"}
            ${package}/bin/cargo-clippy --deny=warnings "$@"
        '';


      inherit
        (config.clippy)
        package
        checks;
      buildClippyCmd = key: cfg: {
        name = name' key;
        value = "${buildClippyCmdScript {
          inherit (config.clippy) package;
          inherit (cfg) subdir src;
        }}/bin/run-clippy.sh";
      };

    in
    {
      steps = lib.mapAttrs' buildClippyStep checks;
      commands = lib.mapAttrs' buildClippyCmd checks;
    };
}
