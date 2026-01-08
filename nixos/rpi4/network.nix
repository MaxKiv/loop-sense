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

  # Prevent host becoming unreachable on wifi after some time
  networking.networkmanager.wifi.powersave = false;

  networking.interfaces.end0.ipv4.addresses = [
    {
      address = "192.168.0.4";
      prefixLength = 24;
    }
  ];

  networking.firewall = {
    enable = true;
    checkReversePath = false;
    allowedTCPPorts = [8000 8086 5173 80];
    allowedUDPPorts = [8000 8086];
  };

}

