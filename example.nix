{ pkgs, lib, config, ... }:

{
  steps.hello.config = {
    label = "Hello world!";
    depends_on = "build-package-hello";
  };

  commands.hello = ''${lib.getBin pkgs.perl}/bin/perl -e "print('hello world')"'';
}
