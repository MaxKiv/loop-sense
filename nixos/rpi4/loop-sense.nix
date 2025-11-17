{
  inputs,
  config,
  username,
  hostname,
  pkgs,
  lib,
  sshPublicKeys,
  loopSensePackage,
  ...
}: {
  # Systemd service to run the Loop Sense application
  systemd.services.loop-sense = {
    description = "Loop Sense Heart Pump Control Application";
    after = ["network-online.target"];
    wants = ["network-online.target"];
    wantedBy = ["multi-user.target"];

    serviceConfig = {
      Type = "simple";
      ExecStart = "${loopSensePackage}/bin/loop_sense";
      Restart = "always";
      RestartSec = "10s";
      
      # Run as max user with access to UART/serial devices
      User = username;
      Group = "dialout";
      
      # Environment variables (if needed)
      # Environment = "RUST_LOG=info";
      
      # Working directory
      WorkingDirectory = "/home/${username}";
      
      # Security hardening (optional)
      # NoNewPrivileges = true;
      # PrivateTmp = true;
    };
  };
}

