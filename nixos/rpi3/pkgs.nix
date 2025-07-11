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
    duf
    neovim
    wget
    ripgrep
  ];
}
