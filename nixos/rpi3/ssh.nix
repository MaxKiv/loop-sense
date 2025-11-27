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
  # Enable openSSH service
  services.openssh.enable = true;

  # Add authorized keys
  users.users.${username} = {
    openssh.authorizedKeys.keys = [
      sshPublicKeys.users.max
      sshPublicKeys.users.kousheek
      sshPublicKeys.users.camille
    ];
  };
  users.users.root = {
    openssh.authorizedKeys.keys = [
      sshPublicKeys.users.max
      sshPublicKeys.users.kousheek
      sshPublicKeys.users.camille
    ];
  };

  # Slight hardening
  services.openssh.settings = {
    PermitRootLogin = "prohibit-password";
    PasswordAuthentication = false;
  };
}

