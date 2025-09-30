use std::time::Instant;

#[cfg(feature = "serde")]
use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NetworkSpeed {
	pub upload_bytes_per_sec: u64,
	pub download_bytes_per_sec: u64,
	pub timestamp: Instant,
}

impl NetworkSpeed {
	pub fn new(upload: u64, download: u64) -> Self {
		Self {
			upload_bytes_per_sec: upload,
			download_bytes_per_sec: download,
			timestamp: Instant::now(),
		}
	}

	pub fn zero() -> Self {
		Self::new(0, 0)
	}

	pub fn upload_kbps(&self) -> f64 {
		((self.upload_bytes_per_sec as f64) * 8.0) / 1_000.0
	}

	pub fn download_kbps(&self) -> f64 {
		((self.download_bytes_per_sec as f64) * 8.0) / 1_000.0
	}

	pub fn upload_mbps(&self) -> f64 {
		((self.upload_bytes_per_sec as f64) * 8.0) / 1_000_000.0
	}

	pub fn download_mbps(&self) -> f64 {
		((self.download_bytes_per_sec as f64) * 8.0) / 1_000_000.0
	}

	pub fn upload_gbps(&self) -> f64 {
		((self.upload_bytes_per_sec as f64) * 8.0) / 1_000_000_000.0
	}

	pub fn download_gbps(&self) -> f64 {
		((self.download_bytes_per_sec as f64) * 8.0) / 1_000_000_000.0
	}

	pub fn upload_formatted(&self) -> String {
		format_bytes_per_second(self.upload_bytes_per_sec)
	}

	pub fn download_formatted(&self) -> String {
		format_bytes_per_second(self.download_bytes_per_sec)
	}

	pub fn upload_bits_formatted(&self) -> String {
		format_bits_per_second(self.upload_bytes_per_sec * 8)
	}

	pub fn download_bits_formatted(&self) -> String {
		format_bits_per_second(self.download_bytes_per_sec * 8)
	}

	pub fn total_bytes_per_sec(&self) -> u64 {
		self.upload_bytes_per_sec.saturating_add(self.download_bytes_per_sec)
	}

	pub fn is_active(&self, threshold_bytes_per_sec: u64) -> bool {
		self.total_bytes_per_sec() > threshold_bytes_per_sec
	}
}

impl Default for NetworkSpeed {
	fn default() -> Self {
		Self::zero()
	}
}

#[derive(Debug, Clone)]
pub struct InterfaceStats {
	pub bytes_sent: u64,
	pub bytes_received: u64,
	pub last_update: Instant,
}

impl InterfaceStats {
	pub fn new(sent: u64, received: u64) -> Self {
		Self {
			bytes_sent: sent,
			bytes_received: received,
			last_update: Instant::now(),
		}
	}

	pub fn total_bytes(&self) -> u64 {
		self.bytes_sent.saturating_add(self.bytes_received)
	}
}

impl Default for InterfaceStats {
	fn default() -> Self {
		Self::new(0, 0)
	}
}

pub fn format_bytes_per_second(bytes_per_sec: u64) -> String {
	const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s", "TB/s"];
	let mut size = bytes_per_sec as f64;
	let mut unit_index = 0;

	while size >= 1024.0 && unit_index < UNITS.len() - 1 {
		size /= 1024.0;
		unit_index += 1;
	}

	if unit_index == 0 {
		format!("{} {}", bytes_per_sec, UNITS[unit_index])
	} else {
		format!("{:.2} {}", size, UNITS[unit_index])
	}
}

pub fn format_bits_per_second(bits_per_sec: u64) -> String {
	const UNITS: &[&str] = &["bps", "Kbps", "Mbps", "Gbps", "Tbps"];
	let mut size = bits_per_sec as f64;
	let mut unit_index = 0;

	while size >= 1000.0 && unit_index < UNITS.len() - 1 {
		size /= 1000.0;
		unit_index += 1;
	}

	if unit_index == 0 {
		format!("{} {}", bits_per_sec, UNITS[unit_index])
	} else {
		format!("{:.2} {}", size, UNITS[unit_index])
	}
}
