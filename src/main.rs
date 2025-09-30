#![cfg(feature = "cli")]

use std::env;
use std::thread;
use std::time::Duration;

use chrono::Local;
use network_speed::types::format_bytes_per_second;
use network_speed::{ list_interfaces, NetworkMonitor, NetworkMonitorConfig };

fn main() {
	let mut args = env::args();
	let _binary = args.next();
	match args.next().as_deref() {
		Some("list") => list_interfaces_command(),
		Some("monitor") | None => monitor_command(),
		Some("help") | Some("--help") | Some("-h") => print_help(),
		Some(other) => {
			eprintln!("Unknown command: {other}");
			print_help();
		}
	}
}

fn print_help() {
	println!("Network Speed Monitor");
	println!("Usage: cargo run --features cli --bin network-speed [COMMAND]");
	println!();
	println!("Commands:");
	println!("  monitor    Monitor network speed (default)");
	println!("  list       List all network interfaces");
	println!("  help       Show this help message");
}

fn list_interfaces_command() {
	println!("Discovered Network Interfaces:");
	println!("{:-<100}", "");

	match list_interfaces() {
		Ok(interfaces) => {
			for iface in interfaces {
				let status_icon = if iface.is_operational { "🟢" } else { "⚪" };
				println!(
					"{status_icon} #{:<3} {:<40} {:<10} {}",
					iface.index,
					iface.description.trim(),
					iface.type_name(),
					iface.formatted_speed()
				);
				println!(
					"    Flags: virtual={}, loopback={}, bluetooth={}",
					iface.is_virtual(),
					iface.is_loopback(),
					iface.is_bluetooth()
				);
			}
		}
		Err(e) => {
			eprintln!("Error listing interfaces: {e}");
		}
	}
}

fn monitor_command() {
	println!("Network Speed Monitor — press Ctrl+C to stop");
	println!("{:-<80}", "");

	let config = NetworkMonitorConfig::builder()
		.exclude_virtual(true)
		.exclude_loopback(true)
		.exclude_bluetooth(true)
		.min_measurement_interval(Duration::from_millis(500))
		.build()
		.expect("valid monitor configuration");

	let mut monitor = NetworkMonitor::with_config(config);

	if let Err(err) = monitor.measure_speed() {
		eprintln!("Initial measurement failed: {err}");
		return;
	}

	println!("Warm-up...");
	thread::sleep(Duration::from_secs(1));

	loop {
		match monitor.measure_speed() {
			Ok(speed) => {
				let timestamp = Local::now().format("%H:%M:%S");
				println!(
					"[{timestamp}] ↑ {:<10} ↓ {:<10} Σ {}",
					speed.upload_formatted(),
					speed.download_formatted(),
					format_bytes_per_second(speed.total_bytes_per_sec())
				);
			}
			Err(err) => {
				eprintln!("Measurement error: {err}");
			}
		}

		thread::sleep(Duration::from_secs(1));
	}
}
