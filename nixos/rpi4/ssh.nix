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
    ];
  };
  users.users.root = {
    openssh.authorizedKeys.keys = [
      sshPublicKeys.users.max
      sshPublicKeys.users.kousheek
    ];
  };

  # Slight hardening
  services.openssh.settings = {
    PermitRootLogin = "prohibit-password";
    PasswordAuthentication = false;
  };
}

