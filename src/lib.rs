#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/DOCS.md"))]
#![doc(html_root_url = "https://docs.rs/network-speed")]

pub mod monitor;
pub mod types;

pub use monitor::*;
pub use types::*;

pub use monitor::sync_monitor::{ NetworkMonitor, NetworkSpeedTracker };

#[cfg(feature = "async")]
pub use monitor::async_monitor::{ AsyncNetworkMonitor, AsyncNetworkSpeedTracker };

pub fn create_monitor() -> NetworkMonitor {
	NetworkMonitor::new()
}

pub fn create_monitor_with_config(config: NetworkMonitorConfig) -> NetworkMonitor {
	NetworkMonitor::with_config(config)
}

pub fn create_speed_tracker(max_history_size: usize) -> NetworkSpeedTracker {
	NetworkSpeedTracker::new(max_history_size)
}

pub fn create_speed_tracker_with_config(config: NetworkMonitorConfig, max_history_size: usize) -> NetworkSpeedTracker {
	NetworkSpeedTracker::with_config(config, max_history_size)
}

#[cfg(feature = "async")]
pub fn create_async_monitor() -> AsyncNetworkMonitor {
	AsyncNetworkMonitor::new()
}

#[cfg(feature = "async")]
pub fn create_async_monitor_with_config(config: NetworkMonitorConfig) -> AsyncNetworkMonitor {
	AsyncNetworkMonitor::with_config(config)
}

#[cfg(feature = "async")]
pub fn create_async_speed_tracker(max_history_size: usize) -> AsyncNetworkSpeedTracker {
	AsyncNetworkSpeedTracker::new(max_history_size)
}

#[cfg(feature = "async")]
pub fn create_async_speed_tracker_with_config(
	config: NetworkMonitorConfig,
	max_history_size: usize
) -> AsyncNetworkSpeedTracker {
	AsyncNetworkSpeedTracker::with_config(config, max_history_size)
}

pub fn list_interfaces() -> Result<Vec<NetworkInterface>> {
	monitor::interface::list_all_interfaces()
}

pub fn get_interface_count() -> Result<usize> {
	monitor::interface::get_interface_count()
}
