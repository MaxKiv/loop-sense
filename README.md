- [0] add control loop
- [ ] communicate with nidaq
- [x] expand streamlit to fetch data from http requests

# Loop-Sense

This is a Rust application to read out sensors from and send setpoints to a
microcontroller connected to a heart mockloop with hydraulic & pneumatic sensors
and actuators. It is designed to run on x86 linux/windows or arm64-linux
(raspberry pi).

It uses the Tokio as async runtime to efficiently divide work done by these 3
different tasks:

    - HTTP Server
    - Control loop
    - Microcontroller I/O

The controller is defined by `MockloopController` which is generic over the
`MockloopHardware` trait, so it is able to function as a controller for any
types implementing it.

# Architecture

`            +----------------+
            |   HTTP Server  |
            | (telemetry/UI) |
            +--------+-------+
                     |
                     v
+---------+    +-----+-----+     +---------------+
| Control |<-->| Channel(s) |<-->| Micro I/O Task|
|  Loop   |    +-----------+     +---------------+
+---------+`
