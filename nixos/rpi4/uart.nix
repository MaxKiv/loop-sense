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
  # Add users to the required usergroups
  users.users = {
    ${username} = {
      extraGroups = ["dialout" "tty" "spi"];
    };
    "root" = {
      extraGroups = ["dialout" "tty" "spi"];
    };
  };

  # Bluetooth is mapped to the preferred UART (/dev/ttyAMA0) by default, disable that
  # hardware.deviceTree.overlays = [
  #   { name = "pi3-disable-bt"; }
  # ];

  # Enable serial console (UART0) TTY
  # boot.kernelParams = [ "console=ttyAMA0,115200n8" ];

  # Additional system packages
  environment.systemPackages = with pkgs; [
    tio
  ];
}
