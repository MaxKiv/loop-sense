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
  virtualisation.docker.enable = true;
  users.users.${username}.extraGroups = ["docker"]; # give user control access to the docker, which equates to root. Very secure

  # Additional system packages
  environment.systemPackages = with pkgs; [
    docker-compose
  ];
}
