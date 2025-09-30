use network_speed::{
	NetworkMonitorConfig,
	NetworkSpeed,
	InterfaceStats,
	format_bytes_per_second,
	format_bits_per_second,
};
use std::time::Duration;

#[test]
fn test_network_speed_creation() {
	let speed = NetworkSpeed::new(1024, 2048);
	assert_eq!(speed.upload_bytes_per_sec, 1024);
	assert_eq!(speed.download_bytes_per_sec, 2048);
	assert_eq!(speed.total_bytes_per_sec(), 3072);
}

#[test]
fn test_speed_conversions() {
	let speed = NetworkSpeed::new(1_000_000, 2_000_000);

	assert!((speed.upload_mbps() - 8.0).abs() < 0.01);
	assert!((speed.download_mbps() - 16.0).abs() < 0.01);
}

#[test]
fn test_formatting() {
	assert_eq!(format_bytes_per_second(1024), "1.00 KB/s");
	assert_eq!(format_bytes_per_second(1048576), "1.00 MB/s");
	assert_eq!(format_bits_per_second(8000), "8.00 Kbps");
	assert_eq!(format_bits_per_second(8_000_000), "8.00 Mbps");
}

#[test]
fn test_is_active() {
	let speed = NetworkSpeed::new(100, 200);
	assert!(speed.is_active(250));
	assert!(!speed.is_active(350));
}

#[test]
fn test_config_default() {
	let config = NetworkMonitorConfig::default();
	assert!(config.exclude_virtual);
	assert!(config.exclude_loopback);
	assert!(config.exclude_bluetooth);
	assert_eq!(config.min_measurement_interval, Duration::from_millis(100));
}

#[test]
fn test_config_builder() {
	let config = NetworkMonitorConfig::builder()
		.exclude_virtual(false)
		.exclude_loopback(false)
		.min_measurement_interval(Duration::from_millis(500))
		.add_interface_name_filter("eth0".to_string())
		.build()
		.unwrap();

	assert!(!config.exclude_virtual);
	assert!(!config.exclude_loopback);
	assert_eq!(config.min_measurement_interval, Duration::from_millis(500));
	assert_eq!(config.interface_name_filters.len(), 1);
}

#[test]
fn test_config_validation() {
	let result = NetworkMonitorConfig::builder()
		.min_measurement_interval(Duration::from_millis(5))
		.build();

	assert!(result.is_err());
}

#[test]
fn test_config_fluent_api() {
	let config = NetworkMonitorConfig::new()
		.with_exclude_virtual(false)
		.with_min_interval(Duration::from_secs(1))
		.add_interface_filter("wifi".to_string());

	assert!(!config.exclude_virtual);
	assert_eq!(config.min_measurement_interval, Duration::from_secs(1));
	assert_eq!(config.interface_name_filters.len(), 1);
}

#[test]
fn test_interface_stats() {
	let stats = InterfaceStats::new(1000, 2000);
	assert_eq!(stats.bytes_sent, 1000);
	assert_eq!(stats.bytes_received, 2000);
	assert_eq!(stats.total_bytes(), 3000);
}

#[test]
fn test_network_speed_zero() {
	let speed = NetworkSpeed::zero();
	assert_eq!(speed.upload_bytes_per_sec, 0);
	assert_eq!(speed.download_bytes_per_sec, 0);
	assert_eq!(speed.total_bytes_per_sec(), 0);
}

#[test]
fn test_speed_units() {
	let speed = NetworkSpeed::new(1024, 2048);

	assert!((speed.upload_kbps() - 8.192).abs() < 0.01);
	assert!((speed.download_kbps() - 16.384).abs() < 0.01);

	assert!((speed.upload_gbps() - 0.000008192).abs() < 0.000000001);
	assert!((speed.download_gbps() - 0.000016384).abs() < 0.000000001);
}

#[test]
fn test_format_edge_cases() {
	assert_eq!(format_bytes_per_second(0), "0 B/s");
	assert_eq!(format_bytes_per_second(512), "512 B/s");
	assert_eq!(format_bytes_per_second(1536), "1.50 KB/s");
	assert_eq!(format_bits_per_second(0), "0 bps");
	assert_eq!(format_bits_per_second(4096), "4.10 Kbps");
}

#[test]
fn test_speed_formatting_methods() {
	let speed = NetworkSpeed::new(1536, 3072);

	assert_eq!(speed.upload_formatted(), "1.50 KB/s");
	assert_eq!(speed.download_formatted(), "3.00 KB/s");
	assert_eq!(speed.upload_bits_formatted(), "12.29 Kbps");
	assert_eq!(speed.download_bits_formatted(), "24.58 Kbps");
}

#[test]
fn test_interface_helpers() {
	let iface = network_speed::NetworkInterface {
		index: 1,
		interface_type: 6,
		description: "Ethernet Adapter".to_string(),
		is_operational: true,
		bytes_sent: 1_000,
		bytes_received: 2_000,
		speed: 1_000_000,
	};

	assert_eq!(iface.type_name(), "Ethernet");
	assert!(iface.formatted_speed().ends_with("Mbps"));
}
