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
  networking.hostName = hostname;
  networking.networkmanager.enable = true;

  networking.interfaces.enu1u1.ipv4.addresses = [
    {
      address = "192.168.0.2";
      prefixLength = 24;
    }
  ];
}
