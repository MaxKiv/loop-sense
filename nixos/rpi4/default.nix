# https://www.eisfunke.com/posts/2023/nixos-on-raspberry-pi.html
# https://nixos.wiki/wiki/NixOS_on_ARM/Raspberry_Pi_4
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
  imports = [
    inputs.nixos-hardware.nixosModules.raspberry-pi-4
    ./network.nix
    ./user.nix
    ./pkgs.nix
    ./ssh.nix
    ./virt.nix
    ./influxdb.nix
    ./uart.nix
    ./loop-sense.nix
    ./heart-os.nix
  ];

  nix.settings.experimental-features = ["nix-command" "flakes"];

  # Trust users in the wheel (sudo) group, used to rebuild over ssh
  nix.settings.trusted-users = ["@wheel"];

  hardware.enableRedistributableFirmware = true;

  # Raspberry Pi 4 specific hardware configuration
  hardware = {
    raspberry-pi."4".apply-overlays-dtmerge.enable = true;
    deviceTree = {
      enable = true;
      filter = "*rpi-4-*.dtb";
    };
  };

  # Disable console for headless operation (optional, can be enabled if needed)
  # console.enable = false;

  boot.initrd.availableKernelModules = ["usbhid"];

  fileSystems."/" = {
    device = "/dev/disk/by-uuid/44444444-4444-4444-8888-888888888888";
    fsType = "ext4";
  };

  swapDevices = [
    {
      device = "/swapfile";
      size = 4 * 1024; # 4GB swap, size in MB
    }
  ];

  # Additional Raspberry Pi utilities
  environment.systemPackages = with pkgs; [
    libraspberrypi
    raspberrypi-eeprom
  ];

  system.stateVersion = "25.05";
}

