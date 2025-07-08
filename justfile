# Print all available just commands
help:
    @just --list

run:
    cargo run --features sim

# send a test controller setpoint using curl
post:
    curl -X POST http://localhost:8000/setpoint      -H "Content-Type: application/json"      -d "{\"enable\":true,\"heart_rate\":1.3333334,\"pressure\":3000.0,\"loop_frequency\":100.0,\"systole_ratio\":0.42857143}"

# get sensor data using curl
get:
    curl -X GET http://localhost:8000/data

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

# Snapshots the docker volume used by the influxdb3 container
docker-save-db-volume:
    docker run --rm -v loop-sense_influxdb3_data:/data -v $PWD/snapshot:/backup alpine tar czf /backup/influxdb3-data.tar.gz -C /data .
