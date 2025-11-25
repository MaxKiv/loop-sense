{
  inputs,
  config,
  username,
  hostname,
  pkgs,
  lib,
  sshPublicKeys,
  composePath,
  ...
}: {

  # Systemd unit to load ARM64 docker images at boot before compose up
  systemd.services.docker-load-images = {
    description = "Load ARM64 Docker images for InfluxDB stack";
    after = ["docker.service"];
    requires = ["docker.service"];
    before = ["influxdb-stack.service"];
    wantedBy = ["multi-user.target"];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      ExecStart = "${pkgs.docker}/bin/docker load -i /etc/influxdb-stack/influxdb-images-arm64.tar";
    };
  };

  # Systemd unit to autorun docker compose on pi startup
  systemd.services.influxdb-stack = {
    description = "InfluxDB3 stack via Docker Compose";
    after = ["network-online.target" "docker.service"];
    wants = ["network-online.target"];
    requires = ["docker.service"];
    bindsTo = ["docker.service"];
    wantedBy = ["multi-user.target"];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      WorkingDirectory = "/etc/influxdb-stack";
      ExecStart = "${pkgs.docker-compose}/bin/docker-compose up -d";
      ExecStop = "${pkgs.docker-compose}/bin/docker-compose down";
      TimeoutStartSec = 0;
    };
  };

  # Copy the docker compose file to the pi during system build
  environment.etc."influxdb-stack/compose.yml" = {
    source = composePath;
    mode = "0644";
  };
}

