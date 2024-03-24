{
  description = "CI configuration expressed with NixOS modules";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      pkg = import ./.;
      systems = [ "aarch64-darwin" "x86_64-linux" ];
    in
    {
      lib = { inherit (pkg) mkMkCIConfig; };
    } // (flake-utils.lib.eachSystem systems (system:
      let
        pkgs = import nixpkgs { inherit system; };
        pkg = import ./.;

        mkCIConfig = pkg.mkMkCIConfig { inherit pkgs nixpkgs system self; };
      in
      {
        ci = mkCIConfig {
          imports = [ ./example.nix ];
        };

        packages.hello = pkgs.hello;
      }));
}
