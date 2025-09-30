use network_speed::{
	NetworkMonitor,
	NetworkSpeedTracker,
	NetworkMonitorConfig,
	list_interfaces,
	get_interface_count,
};
use std::time::Duration;
use std::thread;

#[cfg(feature = "async")]
use network_speed::{ AsyncNetworkMonitor, AsyncNetworkSpeedTracker };

#[test]
fn test_monitor_creation() {
	let monitor = NetworkMonitor::new();
	assert!(monitor.get_config().exclude_virtual);
}

#[test]
fn test_monitor_with_config() {
	let config = NetworkMonitorConfig::builder()
		.exclude_virtual(false)
		.min_measurement_interval(Duration::from_millis(50))
		.build()
		.unwrap();

	let monitor = NetworkMonitor::with_config(config);
	assert_eq!(
		monitor.get_config().min_measurement_interval,
		Duration::from_millis(50)
	);
	assert!(!monitor.get_config().exclude_virtual);
}

#[test]
fn test_measure_speed() {
	if cfg!(windows) {
		let mut monitor = NetworkMonitor::new();

		let first_measurement = monitor.measure_speed();
		assert!(first_measurement.is_ok());

		let first_speed = first_measurement.unwrap();
		assert_eq!(first_speed.upload_bytes_per_sec, 0);
		assert_eq!(first_speed.download_bytes_per_sec, 0);

		thread::sleep(Duration::from_millis(200));

		let second_measurement = monitor.measure_speed();
		assert!(second_measurement.is_ok());
	}
}

#[test]
fn test_speed_tracker() {
	if cfg!(windows) {
		let mut tracker = NetworkSpeedTracker::new(10);

		let result = tracker.track_speed();
		assert!(result.is_ok());

		assert_eq!(tracker.get_history().len(), 1);
	}
}

#[test]
fn test_blocking_measurement() {
	if cfg!(windows) {
		let mut monitor = NetworkMonitor::new();
		let result = monitor.measure_speed_blocking(Duration::from_millis(100));
		assert!(result.is_ok());
	}
}

#[test]
fn test_config_update() {
	let mut monitor = NetworkMonitor::new();

	let new_config = NetworkMonitorConfig::builder()
		.exclude_bluetooth(false)
		.min_measurement_interval(Duration::from_millis(200))
		.build()
		.unwrap();

	let result = monitor.update_config(new_config);
	assert!(result.is_ok());

	assert_eq!(
		monitor.get_config().min_measurement_interval,
		Duration::from_millis(200)
	);
	assert!(!monitor.get_config().exclude_bluetooth);
}

#[test]
fn test_list_interfaces() {
	if cfg!(windows) {
		let result = list_interfaces();
		assert!(result.is_ok(), "Failed to list interfaces: {:?}", result.err());

		let interfaces = result.unwrap();
		assert!(!interfaces.is_empty(), "No interfaces found");

		for interface in &interfaces {
			println!("Interface: {} - {}", interface.index, interface.description);
		}
	}
}

#[test]
fn test_interface_count() {
	if cfg!(windows) {
		let result = get_interface_count();
		assert!(result.is_ok());
		assert!(result.unwrap() > 0);
	}
}

#[test]
fn test_virtual_interface_detection() {
	use network_speed::NetworkInterface;
	use windows::Win32::NetworkManagement::IpHelper::MIB_IFROW;

	let mut row = MIB_IFROW::default();
	row.dwDescrLen = "VMware Virtual Ethernet Adapter".len() as u32;

	let desc_bytes = b"VMware Virtual Ethernet Adapter";
	for (slot, byte) in row.bDescr.iter_mut().zip(desc_bytes.iter()) {
		*slot = *byte;
	}

	let interface = NetworkInterface::from_mib_ifrow(&row).unwrap();
	assert!(interface.is_virtual());
	assert!(!interface.is_loopback());
	assert!(!interface.is_bluetooth());
}

#[test]
fn test_tracker_history_management() {
	if cfg!(windows) {
		let mut tracker = NetworkSpeedTracker::new(3);

		for _ in 0..5 {
			let _ = tracker.track_speed();
			thread::sleep(Duration::from_millis(50));
		}

		let history = tracker.get_history();
		assert!(history.len() <= 3, "History exceeded maximum size");
	}
}

#[test]
fn test_tracker_statistics() {
	if cfg!(windows) {
		let mut tracker = NetworkSpeedTracker::new(5);

		for _ in 0..3 {
			let _ = tracker.track_speed();
			thread::sleep(Duration::from_millis(100));
		}

		let avg_speed = tracker.get_average_speed(Duration::from_secs(1));
		let peak_speed = tracker.get_peak_speed(Duration::from_secs(1));

		if tracker.get_history().len() > 0 {
			assert!(avg_speed.is_some() || peak_speed.is_some());
		}
	}
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_monitor_creation() {
	let monitor = AsyncNetworkMonitor::new();
	let config = monitor.get_config().await;
	assert!(config.exclude_virtual);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_measure_speed() {
	if cfg!(windows) {
		let monitor = AsyncNetworkMonitor::new();

		let result = tokio::time::timeout(
			Duration::from_secs(5),
			monitor.measure_speed()
		).await;
		assert!(result.is_ok(), "Timeout or error in measure_speed");

		let speed_result = result.unwrap();
		assert!(
			speed_result.is_ok(),
			"Failed to measure speed: {:?}",
			speed_result.err()
		);
	}
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_speed_tracker() {
	let tracker = AsyncNetworkSpeedTracker::new(5);

	if cfg!(windows) {
		let result = tracker.track_speed().await;
		assert!(result.is_ok(), "Failed to track speed: {:?}", result.err());

		let history = tracker.get_history().await;
		assert_eq!(history.len(), 1);
	}
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_collect_samples() {
	if cfg!(windows) {
		let monitor = AsyncNetworkMonitor::new();

		let result = tokio::time::timeout(
			Duration::from_secs(10),
			monitor.collect_samples(3, Duration::from_millis(200))
		).await;

		assert!(result.is_ok(), "Timeout in collect_samples");

		let samples_result = result.unwrap();
		if let Ok(samples) = samples_result {
			assert!(!samples.is_empty(), "No samples collected");
			assert!(samples.len() <= 3, "Too many samples collected");
		}
	}
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_monitor_with_channel() {
	if cfg!(windows) {
		let monitor = AsyncNetworkMonitor::new();

		let rx_result = monitor.monitor_with_channel(
			Duration::from_millis(100),
			10
		).await;
		assert!(rx_result.is_ok(), "Failed to create channel monitor");

		let mut rx = rx_result.unwrap();

		let result = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;
		assert!(result.is_ok(), "Timeout waiting for first sample");
	}
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_config_update() {
	let monitor = AsyncNetworkMonitor::new();

	let new_config = NetworkMonitorConfig::builder()
		.exclude_bluetooth(false)
		.min_measurement_interval(Duration::from_millis(50))
		.build()
		.unwrap();

	let result = monitor.update_config(new_config).await;
	assert!(result.is_ok(), "Failed to update config");

	let updated_config = monitor.get_config().await;
	assert!(!updated_config.exclude_bluetooth);
	assert_eq!(updated_config.min_measurement_interval, Duration::from_millis(50));
}
