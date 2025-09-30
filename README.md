# Network Speed Monitor

A high-performance, modern Rust library for monitoring network interface speeds on Windows.

[![Crates.io](https://img.shields.io/crates/v/network-speed.svg)](https://crates.io/crates/network-speed)
[![Documentation](https://docs.rs/network-speed/badge.svg)](https://docs.rs/network-speed)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Features

- **🚀 High Performance**: Optimized for minimal CPU and memory usage
- **🔄 Sync & Async APIs**: Choose between blocking and non-blocking interfaces
- **🎛️ Configurable Filtering**: Exclude virtual interfaces, loopback, Bluetooth, etc.
- **🛡️ Type Safety**: Strong typing with builder patterns and comprehensive validation
- **📊 Rich Metrics**: Upload/download speeds in various units (bytes, bits, formatted strings)
- **📈 Historical Tracking**: Built-in speed history and statistics
- **🔧 Error Handling**: Detailed error types with recovery information
- **📦 Optional Features**: Serde support for serialization, async runtime integration

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
network-speed = "0.2"

# For async support
network-speed = { version = "0.2", features = ["async"] }

# For serialization support
network-speed = { version = "0.2", features = ["serde"] }
```

### Basic Usage (Synchronous)

```rust
use network_speed::NetworkMonitor;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = NetworkMonitor::new();

    // First measurement establishes baseline (returns zero)
    let _baseline = monitor.measure_speed()?;

    thread::sleep(Duration::from_secs(1));

    // Subsequent measurements show actual speed
    let speed = monitor.measure_speed()?;

    println!("Upload: {} ({:.2} Mbps)",
             speed.upload_formatted(),
             speed.upload_mbps());
    println!("Download: {} ({:.2} Mbps)",
             speed.download_formatted(),
             speed.download_mbps());

    Ok(())
}
```

### Async Usage

```rust
use network_speed::AsyncNetworkMonitor;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = AsyncNetworkMonitor::new();

    let speed = monitor.measure_speed().await?;
    println!("Current speed: {} up, {} down",
             speed.upload_formatted(),
             speed.download_formatted());

    // Collect multiple samples
    let samples = monitor.collect_samples(5, Duration::from_secs(1)).await?;
    println!("Collected {} samples", samples.len());

    Ok(())
}
```

### Custom Configuration

```rust
use network_speed::{NetworkMonitor, NetworkMonitorConfig};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkMonitorConfig::builder()
        .exclude_virtual(false)    // Include virtual interfaces
        .exclude_bluetooth(true)   // Exclude Bluetooth interfaces
        .min_measurement_interval(Duration::from_millis(50))
        .add_interface_name_filter("vmware".to_string()) // Filter specific interfaces
        .build()?;

    let mut monitor = NetworkMonitor::with_config(config);
    let speed = monitor.measure_speed()?;

    println!("Speed with custom config: {}", speed.download_formatted());
    Ok(())
}
```

### Speed Tracking with History

```rust
use network_speed::NetworkSpeedTracker;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tracker = NetworkSpeedTracker::new(10); // Keep last 10 measurements

    // Track speed over time
    for _ in 0..5 {
        let speed = tracker.track_speed()?;
        println!("Current: {}", speed.download_formatted());
        std::thread::sleep(Duration::from_secs(1));
    }

    // Get statistics
    let history = tracker.get_history();
    println!("Collected {} measurements", history.len());

    if let Some(avg_speed) = tracker.get_average_speed(Duration::from_secs(10)) {
        println!("Average speed: {}", avg_speed.download_formatted());
    }

    if let Some(peak_speed) = tracker.get_peak_speed(Duration::from_secs(10)) {
        println!("Peak speed: {}", peak_speed.download_formatted());
    }

    Ok(())
}
```

## Interface Detection and Filtering

The library automatically detects and can filter various types of network interfaces:

- **Physical interfaces**: Ethernet, Wi-Fi
- **Virtual interfaces**: VPN, VMware, VirtualBox, Hyper-V
- **System interfaces**: Loopback, Teredo tunneling, IP-HTTPS
- **Bluetooth interfaces**: Bluetooth PAN devices

### Listing Available Interfaces

```rust
use network_speed;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interfaces = network_speed::list_interfaces()?;

    for interface in interfaces {
        println!("Interface {}: {}", interface.index, interface.description);
        println!("  Type: {}, Active: {}, Virtual: {}",
                 interface.interface_type,
                 interface.is_operational,
                 interface.is_virtual());
    }

    Ok(())
}
```

## Advanced Features

### Continuous Monitoring

```rust
use network_speed::AsyncNetworkMonitor;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = AsyncNetworkMonitor::new();

    // Monitor with channel-based updates
    let mut rx = monitor.monitor_with_channel(Duration::from_secs(1), 100).await?;

    while let Some(result) = rx.recv().await {
        match result {
            Ok(speed) => println!("Speed: {}", speed.download_formatted()),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

### Speed Formatting

```rust
use network_speed::NetworkSpeed;

let speed = NetworkSpeed::new(1048576, 2097152); // 1 MB/s up, 2 MB/s down

// Various formatting options
println!("Bytes: {} up, {} down", speed.upload_formatted(), speed.download_formatted());
println!("Bits: {} up, {} down", speed.upload_bits_formatted(), speed.download_bits_formatted());
println!("Mbps: {:.2} up, {:.2} down", speed.upload_mbps(), speed.download_mbps());
println!("Total: {}", speed.total_bytes_per_sec());
println!("Active: {}", speed.is_active(1024)); // Active if > 1KB/s
```

### Inspect Interfaces

```rust
use network_speed::list_interfaces;

fn main() -> network_speed::Result<()> {
    for iface in list_interfaces()? {
        println!(
            "#{} {:<30} {:<10} {} (status: {})",
            iface.index,
            iface.description.trim(),
            iface.type_name(),
            iface.formatted_speed(),
            if iface.is_operational { "Up" } else { "Down" }
        );
    }

    Ok(())
}
```

## System Tray Integration

For building system tray applications, see `examples/tray_monitor.rs` for a complete architecture example showing:

- Background monitoring thread
- Inter-thread communication
- Tray icon updates
- Context menu handling
- Configuration management

## Error Handling

The library provides detailed error information:

```rust
use network_speed::{NetworkMonitor, NetworkError};

let mut monitor = NetworkMonitor::new();

match monitor.measure_speed() {
    Ok(speed) => println!("Speed: {}", speed.download_formatted()),
    Err(NetworkError::InsufficientTimeElapsed { min_ms, actual_ms }) => {
        println!("Too soon! Need at least {}ms, got {}ms", min_ms, actual_ms);
    }
    Err(NetworkError::NoInterfacesFound) => {
        println!("No suitable network interfaces found");
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Configuration Options

- `exclude_virtual`: Filter out virtual network adapters (default: true)
- `exclude_loopback`: Filter out loopback interfaces (default: true)
- `exclude_bluetooth`: Filter out Bluetooth interfaces (default: true)
- `min_measurement_interval`: Minimum time between measurements (default: 100ms)
- `interface_name_filters`: Exclude interfaces containing specific strings
- `interface_type_filters`: Exclude specific interface types by ID

## Performance

- **Memory usage**: < 1MB typical, minimal allocations during operation
- **CPU usage**: < 0.1% on modern systems for 1-second intervals
- **Measurement accuracy**: Millisecond precision, byte-level accuracy
- **Interface detection**: Cached with manual refresh capability

## Windows Compatibility

Requires Windows Vista or later. Uses Windows IP Helper API for maximum compatibility and performance.

Tested on:

- Windows 10/11 (x64)
- Windows Server 2016/2019/2022
- Various network interface types (Ethernet, Wi-Fi, VPN)

## Examples

Run the examples to see the library in action:

```bash
# Basic synchronous usage
cargo run --example basic_usage

# Friendly terminal monitor
cargo run --example monitor

# System tray architecture example
cargo run --example tray_monitor

# With async features
cargo run --example basic_usage --features async

# Optional CLI binary (requires feature)
cargo run --features cli --bin network-speed monitor
```

## Optional Features

- `async`: Enables async/await API with Tokio integration
- `serde`: Enables serialization/deserialization of network speed data
- `cli`: Builds the optional CLI binary with friendly terminal output
