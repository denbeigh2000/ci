{ pkgs, lib, config, ... }:

let
  inherit (pkgs) system;
  inherit (lib) types;

  # Ensure that our steps and commands match 1:1.
  # Put steps in an unsorted list, with their priority attached.
  # Add fields we want to calculate for buildkite (command and step key).
  evaledSteps =
    let
      evalStep = key: step:
        {
          # Keep priority (needed for sorting)
          inherit (step) priority;
          # Inject fields we want to calculate for buildkite
          config = step.config // {
            inherit key;
            command = config.commandBuilder key;
          };
        };
    in
    lib.mapAttrsToList evalStep config.steps;

  # Sort by priority
  sortedSteps = builtins.sort (p: q: p.priority < q.priority) evaledSteps;
  # Remove the priority, just taking the final buildkite-formatted steps
  finalSteps = builtins.map (s: s.config) sortedSteps;

  derivsToBuild = [
    {
      path = ".#packages.${system}.hello";
      # Not sure if this would ever not be /nix/store/?
      hash = (pkgs.lib.removePrefix "/nix/store/" pkgs.hello.outPath);
    }
  ];
in
{
  options = {
    commandBuilder = lib.mkOption {
      type = types.functionTo types.str;
      # TODO: How do we best expose the tool for easy consumption?
      default = (stepKey: "nix run .#tool -- execute ci.${system}.config.commandTargets.${stepKey}");
    };

    steps = lib.mkOption {
      type = types.attrsOf
        (types.submodule {
          options = {
            priority = lib.mkOption {
              type = (types.ints.between 0 100);
              default = 50;
              description = ''
                Priority of this step (relative to all configured steps)
              '';
            };

            config = lib.mkOption {
              # TODO: Not sure if I want to use the type system to validate
              # buildkite steps
              type = types.attrs;
              description = ''
                Config to add to uploaded buildkite pipeline
                (will have key overridden)
              '';
            };
          };
        });

      default = { };
    };

    commands = lib.mkOption {
      # type = types.attrsOf types.str;
      # type = types.attrsOf types.attrs;
      default = { };
      type = types.attrs;
      description = ''
        Commands to execute on the agent, should match 1:1 with steps above.
        Substitute deps in here to depend on them!
      '';
    };

    evaluation = {
      steps = lib.mkOption {
        type = types.listOf types.attrs;
        default = [ ];
        description = ''
          Raw steps to upload to buildkite
        '';
      };

      builds = lib.mkOption {
        type = types.attrsOf types.attrs;
        description = ''
          Contains metadata for all top-level derivations to build
        '';
      };
    };

    commandTargets = lib.mkOption {
      type = types.attrsOf types.package;
      default = { };
      description = ''
        Targets built for running CI commands
      '';
    };
  };

  config = {
    evaluation.steps = finalSteps;
    commandTargets =
      lib.mapAttrs (key: cmd: pkgs.writeScriptBin "run-${key}.sh" cmd) config.commands;
  };
}
