{ config, pkgs, lib, event, ... }:

let
  inherit (lib) types;
in
{
  options = {
    rustfmt = {
      package = lib.mkOption {
        type = types.package;
        default = pkgs.rustfmt;
        description = ''
          The derivation containing the `rustfmt` tool.
        '';
      };

      checks = lib.mkOption {
        description = ''
          The set of `rustfmt` checks to run.
        '';

        default = { };

        type = types.attrsOf (types.submodule {
          options = {
            enable = lib.mkOption {
              type = types.bool;
              default = true;
              description = ''
                If true (the default) this rustfmt check will be run.
              '';
            };

            edition = lib.mkOption {
              type = types.enum [ "2015" "2018" "2021" ];
              default = "2021";
              description = ''
                Edition of rust checks to run.
              '';
            };

            src = lib.mkOption {
              type = types.path;
              description = ''
                Path to the source directory to run on
                (will have /**/*.rs appended)
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
      name' = key: "rustfmt-${key}";
      buildFmtStep = key: cfg: ({
        name = name' key;
        value = {
          config = {
            type = "command";
            label = ":rust: :broom: fmt ${key}";
          };
        };
      });

      inherit (pkgs) findutils writeShellScriptBin;
      inherit (config.rustfmt) package checks;

      fmtTool = edition: (writeShellScriptBin "run-rustfmt.sh" ''
        ${findutils}/bin/find "$@" -name "*.rs" -print0 \
        | ${findutils}/bin/xargs -0 \
          ${package}/bin/rustfmt --check --edition ${edition}
      '');

      buildFmtCmd = key: cfg: {
        name = name' key;
        value = "${fmtTool cfg.edition}/bin/run-rustfmt.sh ${cfg.src}";
      };

    in
    {
      steps = lib.mapAttrs' buildFmtStep checks;
      commands = lib.mapAttrs' buildFmtCmd checks;
    };
}


