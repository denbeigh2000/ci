{ naersk, buildInputs, stdenvNoCC }:

let
  build = pname: naersk.buildPackage {
    inherit buildInputs pname;
    src = ./.;

  };

in
{
  server = build "server";
  tool = build "ci";
}
