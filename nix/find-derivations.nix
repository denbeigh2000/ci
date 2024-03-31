{ self, pkgs, config, ... }:
let
  inherit (pkgs) lib system;
  nullOr = first: second: if first != null then first else second;

  derivName = key: deriv: buildType: (
    ({
      "package" = if deriv ? pname then deriv.pname else deriv.name;
      "devshell" = key;
      "home" = key;
      "nixos" = deriv.config.networking.hostName;
      "darwin" = deriv.config.networking.hostName;
    }).${buildType}
  );

  fmtDeriv = { deriv, name, tag, buildType }: {
    inherit tag;
    build_type = buildType;
    name = derivName name deriv buildType;
    path = deriv.outPath;
  };

  mapSet = typeName:
    (name: value: lib.nameValuePair "${typeName}-${name}" (fmtDeriv value));

  mapPackage = { name, value, tag, typeName }:
    let
      key = "${typeName}-${name}";
      data = fmtDeriv { inherit name tag; deriv = value; buildType = typeName; };
    in
    lib.nameValuePair key data;

  devShells' = if self ? devShells then self.devShells else { };
  devShells = if devShells' ? "${system}" then devShells'.${system} else { };

  packages' = if self ? packages then self.packages else { };
  sysPackages = if packages' ? ${system} then packages'.${system} else { };

  packages = lib.mapAttrs'
    (name: value: mapPackage {
      inherit name value;
      tag = "packages.${system}.${name}";
      typeName = "package";
    })
    sysPackages;


  # First, filter out configurations we can't build
  nixosConfigs' = lib.filterAttrs (n: v: v.pkgs.system == system)
    (if self ? nixosConfigurations then self.nixosConfigurations else { });
  nixosConfigs = lib.mapAttrs'
    (name: value: mapPackage {
      inherit value;
      name = value.config.networking.hostName;
      tag = "nixosConfigurations.${name}.system.build.toplevel";
      typeName = "nixos";
    })
    nixosConfigs';

  darwinConfigs = lib.mapAttrs'
    (name: value: mapPackage {
      inherit name value;
      tag = "darwinConfigurations.${name}.system";
      typeName = "darwin";
    })
    (if self ? darwinConfigurations then self.darwinConfigurations else { });

  devShellConfigs = lib.mapAttrs'
    (name: value: mapPackage {
      inherit name value;
      tag = "devShells.${system}.${name}";
      typeName = "devshell";
    })
    devShells;
in
{
  evaluation.builds = packages // devShellConfigs // nixosConfigs // darwinConfigs;
}

