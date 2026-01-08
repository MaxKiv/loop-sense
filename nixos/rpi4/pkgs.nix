{
  inputs,
  config,
  username,
  hostname,
  pkgs,
  lib,
  sshPublicKeys,
  ...
}: {
  # Default system packages
  environment.systemPackages = with pkgs; [
    git
    duf
    neovim
    wget
    ripgrep
    eza
    nodejs_24
    nginx
  ];
}

