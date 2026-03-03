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

  networking.interfaces.end0.ipv4.addresses = [
    {
      address = "192.168.0.4";
      prefixLength = 24;
    }
  ];

  # DHCP server for the laptop on end0
  services.dnsmasq = {
    enable = true;
    settings = {
      interface = "end0";
      bind-interfaces = true;

      # Hand out addresses in same subnet, excluding .4
      dhcp-range = "192.168.0.50,192.168.0.150,255.255.255.0,12h";

      # Optional: tell clients the gateway is the Pi (useful if you later enable NAT)
      dhcp-option = [
        "option:router,192.168.0.4"
        # Optional DNS advertised to client:
        # "option:dns-server,192.168.0.4"
      ];
    };
  };

  networking.firewall = {
    enable = true;
    checkReversePath = false;
    allowedTCPPorts = [8000 8086 5173 80];
    allowedUDPPorts = [67 8000 8086];
    # allowedUDPPorts = [67 53 8000 8086];
  };
}
