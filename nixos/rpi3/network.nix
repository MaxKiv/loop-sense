{
  inputs,
  config,
  username,
  hostname,
  pkgs,
  lib,
  sshPublicKeys,
  ...
}: let
  ssid = "HetBolProtocol";
  pass = "W!f!456Cod3";
in {
  networking.hostName = hostname;
  networking.networkmanager.enable = true;

  networking.networkmanager.ensureProfiles.profiles = {
    borg-wifi = {
      connection.id = "borg-wifi";
      connection.type = "wifi";
      wifi = {
        mode = "infrastructure";
        ssid = ssid;
      };
      wifi-security = {
        auth-alg = "open";
        key-mgmt = "wpa-psk";
        psk = pass;
      };

      ipv4.method = "auto";
      ipv6.method = "auto";
    };
  };
}
