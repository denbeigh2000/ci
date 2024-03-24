{ self, pkgs, config, ... }:

let
  inherit (pkgs) lib system;
  nullOr = first: second: if first != null then first else second;

  fmtDeriv = { deriv, tag }: {
    inherit tag;
    name = deriv.pname or deriv.name;
    path = deriv.outPath;
  };

  mapSet = typeName:
    (name: value: lib.nameValuePair "${typeName}-${name}" (fmtDeriv value));

  mapPackage = { name, value, tag, typeName }:
    let
      data = fmtDeriv { inherit tag; deriv = value; };
    in
    lib.nameValuePair "${typeName}-${name}" data;

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
    (name: value: mapPackage { })
    (if self ? darwinConfigurations then self.darwinConfigurations else { });
in
{
  evaluation.builds = packages // nixosConfigs // darwinConfigs;
}
