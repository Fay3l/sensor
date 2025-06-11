# Sensor Project

## Overview

This project features a Rust-powered backend and a modern web dashboard for real-time monitoring and interaction with multiple sensors:

- **LD2410C** (mmWave radar): Detects presence, movement, and distance across configurable gates.
- **RD03D** (Doppler radar): Tracks up to three targets, providing X/Y position, speed, distance, and angle.
- **TF-Luna** (Lidar): Measures distance in real time (if enabled).
- **TOF200F** (Time-of-Flight sensor): Provides high-precision distance measurements with a wide detection range and fast response time.

The backend connects to sensors via serial ports, parses incoming data, and serves it through HTTP APIs and Server-Sent Events (SSE). The frontend (HTML/JavaScript) delivers live data streams and interactive radar visualizations.

## Architecture

## Project Structure

This project follows standard Rust conventions with Cargo. Here are the main files and folders:

- `src/main.rs`: The entry point of the application. It initializes the server and loads the sensor modules.
- `src/api.rs`: Defines all HTTP routes (HTML and SSE endpoints) and connects the web interface to the sensor logic.
- `src/ld2410c.rs`: Library for the LD2410C sensor. Handles serial communication, commands, and data parsing for the mmWave radar.
- `src/rd03d.rs`: Library for the RD03D sensor. Handles serial communication, commands, and data parsing for the Doppler radar.
- `src/tf_luna.rs`: Library for the TF-Luna Lidar sensor. Handles serial communication and data parsing for the Lidar.
- `templates/`: Contains Askama HTML templates for the web dashboard.
- `Cargo.toml`: Project configuration and dependencies.
- `README.md`: This documentation file.

## Usage

### 1. Hardware Setup

- Connect each sensor to your computer via USB/UART adapter.
- Note the COM port assigned to each sensor (e.g., `COM7`, `COM8` on Windows).

### 2. Installation

**[Clone the repository](#clone-the-repository)**

### 3. Configuration
Edit the serial port names in src/api.rs or your sensor modules to match your hardware (e.g., "COM7", "COM8").
You can adjust baud rates and other settings in the Rust source files.
### 4. Running the Backend
By default, the backend listens on http://localhost:3000.

### 5. Accessing the Dashboard
Open your browser and go to:

http://localhost:2000/rd03d — RD03D radar dashboard
http://localhost:2000/ld2410c — LD2410C radar dashboard
Live data is updated via SSE (Server-Sent Events).

## Sensors

### LD2410C (mmWave Radar)
- Detects presence, movement, and distance of objects/humans.
- Provides gate-based distance segmentation and engineering data.
- Communicates via UART (serial).

### RD03D (Doppler Radar)
- Detects up to 3 moving targets.
- Provides X/Y coordinates (mm), speed (cm/s), distance (mm), and angle (degrees).
- Communicates via UART (serial).

### TF-Luna (Lidar)
- Measures distance using time-of-flight.
- Communicates via UART (serial).

## Troubleshooting

- **No data?** Check your COM port assignments and that no other program is using the port.
- **CORS errors?** Use relative URLs in JS (/rd03d/sse not http://localhost:3000/rd03d/sse).
- **Permission errors?** On Linux, add your user to the dialout group or run as root.


## Clone the repository

```sh
git clone https://github.com/Fay3l/sensor
cd sensor
```

### Author
 ---
Fayel MOHAMED mohamed.fayel@yahoo.com