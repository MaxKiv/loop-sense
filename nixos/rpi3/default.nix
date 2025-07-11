# https://www.eisfunke.com/posts/2023/nixos-on-raspberry-pi.html
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
    inputs.nixos-hardware.nixosModules.raspberry-pi-3
    ./network.nix
    ./user.nix
    ./ssh.nix
  ];

  nix.settings.experimental-features = ["nix-command" "flakes"];

  hardware.enableRedistributableFirmware = true;

  nixpkgs.hostPlatform = "aarch64-linux";

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

  system.stateVersion = "25.05";
}
