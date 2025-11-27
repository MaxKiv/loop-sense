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
  # Systemd service to run the Heart OS React Vite application
  systemd.services.heart-os = {
    description = "Heart OS React Vite Development Server";
    after = ["network-online.target"];
    wants = ["network-online.target"];
    wantedBy = ["multi-user.target"];

    serviceConfig = {
      Type = "simple";
      ExecStart = "${pkgs.nodejs_24}/bin/npm run dev";
      Restart = "always";
      RestartSec = "10s";
      
      # Run as root user (since folder is in /root)
      User = "root";
      Group = "root";
      
      # Working directory
      WorkingDirectory = "/root/heart_os";
      
      # Environment variables
      Environment = "PATH=${pkgs.nodejs_24}/bin:${pkgs.bash}/bin";
    };
  };
}

