{
  description = "CI configuration expressed with NixOS modules";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    let
      systems = [ "aarch64-darwin" "x86_64-linux" ];
      inherit (import ./nix) mkMkCIConfig;
    in
    {
      lib = { };
    } // (flake-utils.lib.eachSystem systems (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        mkCIConfig = mkMkCIConfig { inherit self pkgs; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.libgit2 ];
          packages = [ rustToolchain ];
        };

        packages.hello = pkgs.hello;

        ci = mkCIConfig
          {
            imports = [ ./example.nix ];
          };
      }));
}
