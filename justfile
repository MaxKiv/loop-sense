# Print all available just commands
help:
    @just --list

run:
    cargo run --features sim

check:
    cargo check

# Snapshots the docker volume used by the influxdb3 container
docker-save-db-volume:
    docker run --rm -v loop-sense_influxdb3_data:/data -v $PWD/snapshot:/backup alpine tar czf /backup/influxdb3-data.tar.gz -C /data .

docker-run-db:
    docker compose up

# Manual rpi3 system image build command
rpi-build:
    nix build .#nixosConfigurations.rpi3.config.system.build.toplevel --print-out-paths

# Manual rpi3 system image copy command
rpi-copy:
    nix copy [store-path-from-rpi-build] --to ssh://root@192.168.0.4

# Manual rpi3 system image copy command 2
rpi-ssh-switch:
    ssh -v root@192.168.0.2
    nixos-install --root / --system [store-path-from-rpi-build]

# Build rpi3 image, copy to rpi3 and switch system
rpi-switch:
    nix run nixpkgs#nixos-rebuild -- switch --flake .#rpi3 --target-host root@192.168.0.2 --verbose --show-trace

# Manual rpi4 system image build command
rpi4-build:
    nix build .#nixosConfigurations.rpi4.config.system.build.toplevel --print-out-paths

# Manual rpi4 system image copy command
rpi4-copy:
    nix copy [store-path-from-rpi4-build] --to ssh://root@192.168.0.4

# Manual rpi4 system image copy command 2
rpi4-ssh-switch:
    ssh -v root@192.168.0.4
    nixos-install --root / --system [store-path-from-rpi4-build]

# Build rpi4 image, copy to rpi4 and switch system
rpi4-switch:
    nix run nixpkgs#nixos-rebuild -- switch --flake .#rpi4 --target-host root@192.168.0.4 --verbose --show-trace
    
### Debug commands ###

# send a test controller setpoint using curl
post:
    curl -X POST http://192.168.0.4:8000/setpoint      -H "Content-Type: application/json"      -d "{\"enable\":true,\"heart_rate\":1.3333334,\"pressure\":3000.0,\"loop_frequency\":100.0,\"systole_ratio\":0.42857143}"

# get sensor data using curl
get:
    curl -X GET http://192.168.0.4:8181/api/v3/query_sql?db=mockloop_data&q=SELECT * from test_data

file-db:
    influxdb3 serve --bearer-token 3ba49996a6de5af183b4e05326b2e13642c7300540d9e2a0b8908bb62275148dd45ef1f39a867e81709d7da42bda7d57edf9cd0cfa1b864fc00278f5b0c93182 --node-id host01   --object-store file   --data-dir ./.influxdb3

mem-db:
    influxdb3 serve --bearer-token 3ba49996a6de5af183b4e05326b2e13642c7300540d9e2a0b8908bb62275148dd45ef1f39a867e81709d7da42bda7d57edf9cd0cfa1b864fc00278f5b0c93182 --node-id host01 --object-store memory

create-table:
    influxdb3 create database mockloop_data --token apiv3_5zB9k-A7Eora5iMy3epTdWi6NjaRzTvF2jx1mprt98l2z4eOl2tyZTLdnjHzzmqB4kwD_z681ynKVaSXf4Lvcw

# INFLUXDB3_AUTH_TOKEN=apiv3_5zB9k-A7Eora5iMy3epTdWi6NjaRzTvF2jx1mprt98l2z4eOl2tyZTLdnjHzzmqB4kwD_z681ynKVaSXf4Lvcw influxdb3 write --database mockloop_data --token apiv3_5zB9k-A7Eora5iMy3epTdWi6NjaRzTvF2jx1mprt98l2z4eOl2tyZTLdnjHzzmqB4kwD_z681ynKVaSXf4Lvcw --precision ns --accept-partial --file sensor_data.lp && \
write-db:
    INFLUXDB3_AUTH_TOKEN=apiv3_5zB9k-A7Eora5iMy3epTdWi6NjaRzTvF2jx1mprt98l2z4eOl2tyZTLdnjHzzmqB4kwD_z681ynKVaSXf4Lvcw influxdb3 write --database mockloop_data --token apiv3_5zB9k-A7Eora5iMy3epTdWi6NjaRzTvF2jx1mprt98l2z4eOl2tyZTLdnjHzzmqB4kwD_z681ynKVaSXf4Lvcw --precision ns --accept-partial --file measurement_id.lp

show-db database:
    INFLUXDB3_AUTH_TOKEN=apiv3_5zB9k-A7Eora5iMy3epTdWi6NjaRzTvF2jx1mprt98l2z4eOl2tyZTLdnjHzzmqB4kwD_z681ynKVaSXf4Lvcw influxdb3 query --database mockloop_data "SELECT * from {{ database }}"

id-db:
    INFLUXDB3_AUTH_TOKEN=apiv3_5zB9k-A7Eora5iMy3epTdWi6NjaRzTvF2jx1mprt98l2z4eOl2tyZTLdnjHzzmqB4kwD_z681ynKVaSXf4Lvcw influxdb3 query --database mockloop_data "SELECT * from measurement_id"

tables-db:
    INFLUXDB3_AUTH_TOKEN=apiv3_5zB9k-A7Eora5iMy3epTdWi6NjaRzTvF2jx1mprt98l2z4eOl2tyZTLdnjHzzmqB4kwD_z681ynKVaSXf4Lvcw influxdb3 query --database mockloop_data "SHOW TABLES"

# SSH into rpi3
ssh:
    ssh -v root@192.168.0.2

# SSH into rpi4
ssh4:
    ssh -v root@192.168.0.4
