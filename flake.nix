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
      inherit (import ./nix) mkCIConfig;
    in
    {
      lib = { inherit mkCIConfig; };
    } // (flake-utils.lib.eachSystem systems (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchainPkgs = pkgs.rust-bin.stable.latest;
        naersk' = pkgs.callPackage naersk {
          cargo = rustToolchainPkgs.default;
          rustc = rustToolchainPkgs.default;
        };

        buildInputs =
          if pkgs.stdenvNoCC.targetPlatform.isDarwin
          then
            (builtins.attrValues {
              inherit (pkgs) pkg-config openssl;
              inherit (pkgs.darwin.apple_sdk.frameworks) SystemConfiguration;
            })
          else [ ];

        rustPkgs = pkgs.callPackage ./rust.nix {
          inherit buildInputs;
          naersk = naersk';
        };
      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          packages = [ rustToolchainPkgs.default ];
        };

        packages = {
          inherit (rustPkgs) tool server;
        };

        ci = mkCIConfig
          {
            inherit self pkgs;
            config = {
              rustfmt = {
                package = rustToolchainPkgs.rustfmt-preview;
                checks = {
                  tool.src = ./tool/src;
                  server.src = ./server/src;
                };
              };

              clippy = {
                package = rustToolchainPkgs.clippy-preview;
                checks = {
                  tool.src = ./tool/src;
                  server.src = ./server/src;
                };
              };
            };
          };
      }));
}
