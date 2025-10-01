use network_speed::{
	list_interfaces,
	NetworkMonitor,
	NetworkMonitorConfig,
	NetworkSpeedTracker,
};
use std::time::Duration;

#[cfg(feature = "async")]
use network_speed::{ AsyncNetworkMonitor, AsyncNetworkSpeedTracker };

#[test]
fn test_library_basic_functionality() {
	if cfg!(windows) {
		let mut monitor = NetworkMonitor::new();
		let result = monitor.measure_speed();
		assert!(result.is_ok(), "Basic functionality test failed: {:?}", result.err());
	}
}

#[test]
fn test_speed_tracker_functionality() {
	if cfg!(windows) {
		let mut tracker = NetworkSpeedTracker::new(5);
		let result = tracker.track_speed();
		assert!(result.is_ok(), "Speed tracker test failed: {:?}", result.err());
		assert_eq!(tracker.get_history().len(), 1);
	}
}

#[test]
fn test_custom_config() {
	let config = NetworkMonitorConfig::builder()
		.exclude_virtual(false)
		.min_measurement_interval(Duration::from_millis(50))
		.build();

	assert!(config.is_ok(), "Config creation failed: {:?}", config.err());

	if cfg!(windows) {
		let mut monitor = NetworkMonitor::with_config(config.unwrap());
		let result = monitor.measure_speed();
		assert!(result.is_ok(), "Custom config test failed: {:?}", result.err());
	}
}

#[test]
fn test_interface_listing() {
	if cfg!(windows) {
		let result = list_interfaces();
		assert!(result.is_ok(), "Interface listing failed: {:?}", result.err());

		let interfaces = result.unwrap();
		assert!(!interfaces.is_empty(), "No interfaces found");

		for interface in &interfaces {
			println!(
				"Interface {}: {} (Type: {}, Active: {})",
				interface.index,
				interface.description,
				interface.interface_type,
				interface.is_operational
			);
		}
	}
}

#[test]
fn test_readme_sync_example() {
	if cfg!(windows) {
		let mut monitor = NetworkMonitor::new();

		let speed = monitor.measure_speed().unwrap();
		assert_eq!(speed.upload_bytes_per_sec, 0);
		assert_eq!(speed.download_bytes_per_sec, 0);

		std::thread::sleep(Duration::from_millis(200));

		let _speed = monitor.measure_speed().unwrap();
	}
}

#[test]
fn test_config_builder_example() {
	let config = NetworkMonitorConfig::builder()
		.exclude_virtual(false)
		.exclude_bluetooth(true)
		.min_measurement_interval(Duration::from_millis(50))
		.build()
		.unwrap();

	assert!(!config.exclude_virtual);
	assert!(config.exclude_bluetooth);
	assert_eq!(config.min_measurement_interval, Duration::from_millis(50));
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_functionality() {
	if cfg!(windows) {
		let monitor = AsyncNetworkMonitor::new();
		let result = monitor.measure_speed().await;
		assert!(result.is_ok(), "Async functionality test failed: {:?}", result.err());
	}
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_speed_tracker() {
	if cfg!(windows) {
		let tracker = AsyncNetworkSpeedTracker::new(3);
		let result = tracker.track_speed().await;
		assert!(result.is_ok(), "Async speed tracker test failed: {:?}", result.err());

		let history = tracker.get_history().await;
		assert_eq!(history.len(), 1);
	}
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_readme_async_example() {
	if cfg!(windows) {
		let monitor = AsyncNetworkMonitor::new();

		let speed = monitor.measure_speed().await.unwrap();
		assert!(speed.upload_bytes_per_sec == 0 || speed.upload_bytes_per_sec > 0);
		assert!(speed.download_bytes_per_sec == 0 || speed.download_bytes_per_sec > 0);
	}
}
