{
  description = "CI configuration expressed with NixOS modules";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
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

  outputs = { self, nixpkgs, flake-utils, rust-overlay, naersk }:
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
        naersk' = pkgs.callPackage naersk {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        # TODO: improve ux
        mkCIConfig = mkMkCIConfig {
          inherit self pkgs;
        };

        macosRustPkgs = (builtins.attrValues {
          inherit (pkgs) pkg-config openssl;
          inherit (pkgs.darwin.apple_sdk.frameworks) SystemConfiguration;
        });
        rustPkgs = pkgs.callPackage ./rust.nix {
          inherit macosRustPkgs;
          naersk = naersk';
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = macosRustPkgs;
          packages = [ rustToolchain ];
        };

        packages = {
          inherit (pkgs) hello;
          inherit (rustPkgs) tool server;
        };

        ci = mkCIConfig {
          imports = [ ./example.nix ];
        };
      }));
}
