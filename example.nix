{ pkgs, lib, config, ... }:

lib.mkIf true {
  steps.hello.config = {
    label = "Hello world!";
    depends_on = "build-package-hello";
  };

  commands.hello = ''${pkgs.lib.getBin pkgs.perl}/bin/perl -e "print('hello world')"'';
}
