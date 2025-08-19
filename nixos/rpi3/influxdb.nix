{
  inputs,
  config,
  username,
  hostname,
  pkgs,
  lib,
  sshPublicKeys,
  snapshotPath,
  composePath,
  resourcePath,
  ...
}: {
  # Copy docker image archive to the system
  environment.etc."influxdb-stack/influxdb-images.tar".source = "${resourcePath}/influxdb-images.tar";

  # Systemd unit to load docker images at boot before compose up
  systemd.services.docker-load-images = {
    description = "Load Docker images for InfluxDB stack";
    after = ["docker.service"];
    requires = ["docker.service"];
    before = ["influxdb-stack.service"];
    wantedBy = ["multi-user.target"];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      ExecStart = "${pkgs.docker}/bin/docker load -i /etc/influxdb-stack/influxdb-images.tar";
    };
  };

  # Systemd unit to autorun docker compose on pi startup
  systemd.services.influxdb-stack = {
    description = "InfluxDB3 stack via Docker Compose";
    after = ["network-online.target" "docker.service"];
    wants = ["network-online.target"];
    requires = ["docker.service" "docker-load-images.service"];
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

  # Copy the docker compose file and volume snapshots to the pi during system build
  # environment.etc."influxdb-stack/compose.yml".source = filePath;
  environment.etc."influxdb-stack/compose.yml" = {
    source = composePath;
    mode = "0644";
  };
  environment.etc."influxdb-stack/snapshot/influxdb3-data.tar.gz".source = "${snapshotPath}/influxdb3-data.tar.gz";
  environment.etc."influxdb-stack/snapshot/restore-volume.sh".source = "${snapshotPath}/restore-volume.sh";
}
