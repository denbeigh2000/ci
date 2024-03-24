{ self, pkgs, config, ... }:

let
  inherit (pkgs) lib system;
  nullOr = first: second: if first != null then first else second;

  fmtDeriv = { deriv, name, tag }:
    let
      derivName = if deriv ? pname then deriv.pname else deriv.name;
      name' = if derivName == "nix-shell" then name else derivName;
    in
    {
      inherit tag;
      name = name';
      path = deriv.outPath;
    };

  mapSet = typeName:
    (name: value: lib.nameValuePair "${typeName}-${name}" (fmtDeriv value));

  mapPackage = { name, value, tag, typeName }:
    let
      key = "${typeName}-${name}";
      data = fmtDeriv { inherit name tag; deriv = value; };
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
    (if self ? nixosConfigurations then self.nixosConfigrations else { });
  nixosConfigs = lib.mapAttrs'
    (name: value: mapPackage {
      inherit name value;
      tag = "nixosConfigurations.${name}";
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

