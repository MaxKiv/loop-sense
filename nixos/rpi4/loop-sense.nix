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
  # Systemd service to run the Loop Sense application
  systemd.services.loop-sense = {
    description = "Loop Sense Heart Pump Control Application";
    after = ["network-online.target"];
    wants = ["network-online.target"];
    wantedBy = ["multi-user.target"];

    serviceConfig = {
      Type = "simple";
      ExecStart = "/root/loop_sense/target/release/loop_sense";
      Restart = "always";
      RestartSec = "10s";
      
      # Run as root user (since executable is in /root)
      User = "root";
      Group = "root";
      
      # Environment variables (if needed)
      # Environment = "RUST_LOG=info";
      
      # Working directory
      WorkingDirectory = "/root/loop_sense";
      
      # Security hardening (optional)
      # NoNewPrivileges = true;
      # PrivateTmp = true;
    };
  };
}

