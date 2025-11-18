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

  # Copy the docker compose file and volume snapshots to the pi during system build
  # environment.etc."influxdb-stack/compose.yml".source = filePath;
  environment.etc."influxdb-stack/compose.yml" = {
    source = composePath;
    mode = "0644";
  };
  environment.etc."influxdb-stack/snapshot/influxdb3-data.tar.gz".source = "${snapshotPath}/influxdb3-data.tar.gz";
  environment.etc."influxdb-stack/snapshot/restore-volume.sh".source = "${snapshotPath}/restore-volume.sh";
}
