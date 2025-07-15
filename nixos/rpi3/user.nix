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
  users.users.${username} = {
    isNormalUser = true;
    home = "/home/${username}";
    extraGroups = ["wheel"];

    initialHashedPassword = "$y$j9T$FfnFtEfUDFx4Wo2er3nna/$FknAPmQ768FbtWyaJeFWc8UCaf7z6wxK4ohpeS7Yti/";
  };

  # Setup root user
  users.users."root" = {
    # hashedPasswordFile = config.sops.secrets."pass/root".path;
    initialHashedPassword = "$y$j9T$FfnFtEfUDFx4Wo2er3nna/$FknAPmQ768FbtWyaJeFWc8UCaf7z6wxK4ohpeS7Yti/";
  };

  # Only Nix can mutate users
  users.mutableUsers = false;
}
