# `network-speed` Documentation

Welcome! This document collects the usage guides, code examples, configuration options, and advanced
features for the `network-speed` crate. The README now provides a brief overview—refer here whenever
you need deeper explanations or copy-ready snippets.

---

## Table of contents

1. [Overview](#overview)
2. [Quick start](#quick-start)
3. [Usage examples](#usage-examples)
   1. [Synchronous monitoring](#synchronous-monitoring)
   2. [Asynchronous monitoring](#asynchronous-monitoring)
   3. [Custom configuration](#custom-configuration)
   4. [Tracking with history](#tracking-with-history)
4. [Interface inspection & filtering](#interface-inspection--filtering)
5. [Advanced monitoring](#advanced-monitoring)
6. [Formatting helpers](#formatting-helpers)
7. [System tray integration](#system-tray-integration)
8. [Error handling](#error-handling)
9. [Configuration reference](#configuration-reference)
10. [Performance characteristics](#performance-characteristics)
11. [Windows compatibility](#windows-compatibility)
12. [Examples catalog](#examples-catalog)
13. [Optional Cargo features](#optional-cargo-features)

---

## Overview

`network-speed` is a high-performance Rust library focused on Windows systems. It provides ergonomic
APIs—both synchronous and asynchronous—for monitoring network adapters, aggregating per-interface
statistics, and formatting speeds for display.

### Feature highlights

- **🚀 High performance**: Minimal allocations with Windows IP Helper API under the hood.
- **🔄 Sync & async APIs**: Choose blocking or Tokio-powered workflows via feature flags.
- **🎛️ Configurable filtering**: Exclude virtual, loopback, Bluetooth, or custom interface patterns.
- **🛡️ Type safety**: Strongly typed builders, error enums, and speed structs.
- **📊 Rich metrics**: Helpers for bytes, bits, totals, Mbps, and formatted output.
- **📈 Historical tracking**: Built-in speed history with averages and peaks.
- **🔧 Error ergonomics**: Detailed error types with recoverability hints.
- **📦 Optional features**: `async`, `serde`, and `cli` for extra integrations.

---

## Quick start

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
network-speed = "0.2"

# Async support
network-speed = { version = "0.2", features = ["async"] }

# Serialization support
network-speed = { version = "0.2", features = ["serde"] }
```

---

## Usage examples

### Synchronous monitoring

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

### Asynchronous monitoring

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

### Custom configuration

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

### Tracking with history

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

---

## Interface inspection & filtering

The library automatically detects and can filter various types of interfaces:

- **Physical**: Ethernet, Wi-Fi, wired adapters
- **Virtual**: VPN, VMware, VirtualBox, Hyper-V, tunneling
- **System**: Loopback, Teredo, IP-HTTPS, ISATAP
- **Bluetooth**: PAN devices

### Listing available interfaces

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

---

## Advanced monitoring

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

---

## Formatting helpers

```rust
use network_speed::NetworkSpeed;

let speed = NetworkSpeed::new(1_048_576, 2_097_152); // 1 MB/s up, 2 MB/s down

println!("Bytes: {} up, {} down", speed.upload_formatted(), speed.download_formatted());
println!("Bits: {} up, {} down", speed.upload_bits_formatted(), speed.download_bits_formatted());
println!("Mbps: {:.2} up, {:.2} down", speed.upload_mbps(), speed.download_mbps());
println!("Total: {}", speed.total_bytes_per_sec());
println!("Active: {}", speed.is_active(1024)); // Active if > 1 KB/s
```

### Inspect interface helpers

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

---

## System tray integration

For a complete tray-application example, reference `examples/tray_monitor.rs`. It demonstrates:

- Background monitoring threads
- Inter-thread communication with channels
- Tray icon updates and context menu actions
- Configuration management patterns

---

## Error handling

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

---

## Configuration reference

- `exclude_virtual`: Filter out virtual adapters (default: `true`).
- `exclude_loopback`: Filter loopback interfaces (default: `true`).
- `exclude_bluetooth`: Filter Bluetooth adapters (default: `true`).
- `min_measurement_interval`: Smallest allowed interval between measurements (default: `100 ms`).
- `interface_name_filters`: Case-insensitive substrings to exclude specific adapters.
- `interface_type_filters`: Filter by Windows interface type IDs.
- `max_counter_wrap_threshold`: Guard against counter wrap/overflow scenarios.

---

## Performance characteristics

- **Memory usage**: Typically under 1 MB, minimal per-sample allocations.
- **CPU usage**: < 0.1% on modern hardware with 1-second sampling.
- **Measurement accuracy**: Millisecond precision with byte-level counters.
- **Interface detection**: Cached lookups with manual refresh support.

---

## Windows compatibility

Requires Windows Vista or later (x64 recommended). Tested against:

- Windows 10 / 11 (x64)
- Windows Server 2016 / 2019 / 2022
- Ethernet, Wi-Fi, VPN, and various virtual adapters

---

## Examples catalog

```bash
# Basic synchronous usage
cargo run --example basic_usage

# Friendly terminal monitor
cargo run --example monitor

# System tray architecture example
cargo run --example tray_monitor

# With async features enabled
cargo run --example basic_usage --features async

# Optional CLI binary (requires `cli` feature)
cargo run --features cli --bin network-speed monitor
```

---

## Optional Cargo features

- `async`: Enables Tokio-powered asynchronous APIs.
- `serde`: Adds serialization/deserialization for configuration and data types.
- `cli`: Builds the optional CLI binary for quick terminal monitoring.

---

Need something that is not covered here? Open an issue or start a discussion—contributions are always
welcome! Consult `CONTRIBUTING.md` before submitting pull requests.
