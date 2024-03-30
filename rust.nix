{ naersk, macosRustPkgs, lib, stdenvNoCC }:

let
  build = pname: naersk.buildPackage {
    inherit pname;
    src = ./.;

    buildInputs = lib.optional stdenvNoCC.targetPlatform.isDarwin macosRustPkgs;
  };

in
{
  server = build "server";
  tool = build "ci";
}
