# Print all available just commands
help:
    @just --list

# send a test controller setpoint using curl
post:
    curl -X POST http://localhost:8000/setpoint      -H "Content-Type: application/json"      -d "{\"enable\":true,\"heart_rate\":2.3333334,\"pressure\":0.2,\"loop_frequency\":100.0,\"systole_ratio\":0.42857143}"

# get sensor data using curl
get:
    curl -X GET http://localhost:8000/data
