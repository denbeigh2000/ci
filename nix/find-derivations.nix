{ self, pkgs, config, ... }:
let
  inherit (pkgs) lib system;
  nullOr = first: second: if first != null then first else second;

  fmtDeriv = { deriv, displayName, tag, buildType }: {
    inherit tag;
    build_type = buildType;
    name = displayName;
    path = deriv.outPath;
  };

  mapPackage = { displayName, name, value, tag, typeName }:
    let
      key = "${typeName}-${name}";
      data = fmtDeriv { inherit displayName tag; deriv = value; buildType = typeName; };
    in
    lib.nameValuePair key data;

  devShells' = if self ? devShells then self.devShells else { };
  devShells = if devShells' ? "${system}" then devShells'.${system} else { };

  packages' = if self ? packages then self.packages else { };
  sysPackages = if packages' ? ${system} then packages'.${system} else { };

  packages = lib.mapAttrs'
    (name: value: mapPackage {
      inherit name value;
      displayName = name;
      tag = "packages.${system}.${name}";
      typeName = "package";
    })
    sysPackages;


  # First, filter out configurations we can't build
  nixosConfigs' = lib.filterAttrs (n: v: v.pkgs.system == system)
    (if self ? nixosConfigurations then self.nixosConfigurations else { });
  nixosConfigs = lib.mapAttrs'
    (name: value: mapPackage {
      inherit name;
      value = value.config.system.build.toplevel;
      displayName = value.config.networking.hostName;
      tag = "nixosConfigurations.${name}.system.build.toplevel";
      typeName = "nixos";
    })
    nixosConfigs';

  darwinConfigs = lib.mapAttrs'
    (name: value: mapPackage {
      inherit name;
      displayName = value.config.networking.hostName;
      value = value.system;
      tag = "darwinConfigurations.${name}.system";
      typeName = "darwin";
    })
    (if self ? darwinConfigurations then self.darwinConfigurations else { });

  devShellConfigs = lib.mapAttrs'
    (name: value: mapPackage {
      inherit name value;
      displayName = name;
      tag = "devShells.${system}.${name}";
      typeName = "devshell";
    })
    devShells;
in
{
  evaluation.builds = packages // devShellConfigs // nixosConfigs // darwinConfigs;
}

