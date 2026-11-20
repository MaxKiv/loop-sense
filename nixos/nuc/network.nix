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

  networking.networkmanager.unmanaged = [ "interface-name:end0" ];
  # networking.wireless.regdom = "NL"; # Fixes brcmfmac kernel error?

  # Set rpi ip addr on end0 (eth)
  networking.interfaces.end0.ipv4.addresses = [
    {
      address = "192.168.0.4";
      prefixLength = 24;
    }
  ];

  # DHCP server for the laptop on end0
  # This is used to make the pi hand out an ip to any connected device, 
  # but leaves the devices connectivity otherwise unaffected
  services.dnsmasq = {
    enable = true;
    settings = {
      interface = "end0";
      bind-interfaces = true;

      # Hand out addresses in same subnet, excluding .4
      dhcp-range = "192.168.0.50,192.168.0.150,255.255.255.0,12h";
    };
  };

  # RPi4 should not do any ip forwarding, 
  boot.kernel.sysctl = {
    "net.ipv4.ip_forward" = 0;
    "net.ipv6.conf.all.forwarding" = 0;
  };

  # Poke required holes in firewall
  networking.firewall = {
    enable = true;
    checkReversePath = false;
    allowedTCPPorts = [22 8000 8086 5173 80];
    allowedUDPPorts = [67 8000 8086];
  };
}
