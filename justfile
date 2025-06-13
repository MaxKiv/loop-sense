# Print all available just commands
help:
    @just --list

# send a test controller setpoint using curl
post:
    curl -X POST http://localhost:8000/setpoint      -H "Content-Type: application/json"      -d "{\"enable\":true,\"heart_rate\":2.3333334,\"pressure\":0.2,\"loop_frequency\":100.0,\"systole_ratio\":0.42857143}"

# get sensor data using curl
get:
    curl -X GET http://localhost:8000/data

run-db:
    influxdb3 serve --bearer-token 3ba49996a6de5af183b4e05326b2e13642c7300540d9e2a0b8908bb62275148dd45ef1f39a867e81709d7da42bda7d57edf9cd0cfa1b864fc00278f5b0c93182 --node-id host01   --object-store file   --data-dir ./.influxdb3

mem-db:
    influxdb3 serve --bearer-token 3ba49996a6de5af183b4e05326b2e13642c7300540d9e2a0b8908bb62275148dd45ef1f39a867e81709d7da42bda7d57edf9cd0cfa1b864fc00278f5b0c93182 --node-id host01 --object-store memory

show-db:
    influxdb3 query --database test "SELECT * from sensor_data"
