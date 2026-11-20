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
  # Assumes uart-usb converter

  # Add users to the required usergroups
  users.users = {
    ${username} = {
      extraGroups = ["dialout" "tty" ];
    };
    "root" = {
      extraGroups = ["dialout" "tty" ];
    };
  };

  # Additional system packages
  environment.systemPackages = with pkgs; [
    tio # serial monitor
  ];
}
