use std::num::NonZeroU8;
use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{ Deserialize, Serialize };

use super::error::{ NetworkError, Result };

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NetworkMonitorConfig {
	pub exclude_virtual: bool,
	pub exclude_loopback: bool,
	pub exclude_bluetooth: bool,
	pub min_measurement_interval: Duration,
	pub max_counter_wrap_threshold: u64,
	pub interface_name_filters: Vec<String>,
	pub interface_type_filters: Vec<u32>,
	pub include_interface_indices: Vec<u32>,
	pub include_interface_name_patterns: Vec<String>,
	pub precision: PrecisionMode,
}

impl NetworkMonitorConfig {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn builder() -> NetworkMonitorConfigBuilder {
		NetworkMonitorConfigBuilder::new()
	}

	pub fn validate(&self) -> Result<()> {
		if self.min_measurement_interval < Duration::from_millis(10) {
			return Err(NetworkError::InvalidConfiguration {
				field: "min_measurement_interval must be at least 10ms".to_string(),
			});
		}

		if self.max_counter_wrap_threshold == 0 {
			return Err(NetworkError::InvalidConfiguration {
				field: "max_counter_wrap_threshold cannot be zero".to_string(),
			});
		}

		if let PrecisionMode::Samples { samples, .. } = &self.precision {
			if samples.get() < 2 {
				return Err(NetworkError::InvalidConfiguration {
					field: "precision.samples.samples must be >= 2".to_string(),
				});
			}
		}

		self.precision.validate()?;

		Ok(())
	}

	pub fn with_exclude_virtual(mut self, exclude: bool) -> Self {
		self.exclude_virtual = exclude;
		self
	}

	pub fn with_exclude_loopback(mut self, exclude: bool) -> Self {
		self.exclude_loopback = exclude;
		self
	}

	pub fn with_exclude_bluetooth(mut self, exclude: bool) -> Self {
		self.exclude_bluetooth = exclude;
		self
	}

	pub fn with_min_interval(mut self, interval: Duration) -> Self {
		self.min_measurement_interval = interval;
		self
	}

	pub fn add_interface_filter(mut self, filter: String) -> Self {
		self.interface_name_filters.push(filter);
		self
	}

	pub fn add_type_filter(mut self, interface_type: u32) -> Self {
		self.interface_type_filters.push(interface_type);
		self
	}

	pub fn with_include_interface_indices(mut self, indices: Vec<u32>) -> Self {
		self.include_interface_indices = indices;
		self
	}

	pub fn with_include_interface_name_patterns(mut self, patterns: Vec<String>) -> Self {
		self.include_interface_name_patterns = patterns;
		self
	}

	pub fn with_precision(mut self, precision: PrecisionMode) -> Self {
		self.precision = precision;
		self
	}
}

impl Default for NetworkMonitorConfig {
	fn default() -> Self {
		Self {
			exclude_virtual: true,
			exclude_loopback: true,
			exclude_bluetooth: true,
			min_measurement_interval: Duration::from_millis(100),
			max_counter_wrap_threshold: 1u64 << 62,
			interface_name_filters: Vec::new(),
			interface_type_filters: vec![24],
			include_interface_indices: Vec::new(),
			include_interface_name_patterns: Vec::new(),
			precision: PrecisionMode::Instant,
		}
	}
}

pub struct NetworkMonitorConfigBuilder {
	config: NetworkMonitorConfig,
}

impl NetworkMonitorConfigBuilder {
	pub fn new() -> Self {
		Self {
			config: NetworkMonitorConfig::default(),
		}
	}

	pub fn exclude_virtual(mut self, exclude: bool) -> Self {
		self.config.exclude_virtual = exclude;
		self
	}

	pub fn exclude_loopback(mut self, exclude: bool) -> Self {
		self.config.exclude_loopback = exclude;
		self
	}

	pub fn exclude_bluetooth(mut self, exclude: bool) -> Self {
		self.config.exclude_bluetooth = exclude;
		self
	}

	pub fn min_measurement_interval(mut self, interval: Duration) -> Self {
		self.config.min_measurement_interval = interval;
		self
	}

	pub fn max_counter_wrap_threshold(mut self, threshold: u64) -> Self {
		self.config.max_counter_wrap_threshold = threshold;
		self
	}

	pub fn add_interface_name_filter(mut self, filter: impl Into<String>) -> Self {
		self.config.interface_name_filters.push(filter.into());
		self
	}

	pub fn add_interface_type_filter(mut self, interface_type: u32) -> Self {
		self.config.interface_type_filters.push(interface_type);
		self
	}

	pub fn interface_name_filters(mut self, filters: Vec<String>) -> Self {
		self.config.interface_name_filters = filters;
		self
	}

	pub fn interface_type_filters(mut self, filters: Vec<u32>) -> Self {
		self.config.interface_type_filters = filters;
		self
	}

	pub fn include_interface_indices(mut self, indices: Vec<u32>) -> Self {
		self.config.include_interface_indices = indices;
		self
	}

	pub fn include_interface_name_patterns(mut self, patterns: Vec<String>) -> Self {
		self.config.include_interface_name_patterns = patterns;
		self
	}

	pub fn precision(mut self, precision: PrecisionMode) -> Self {
		self.config.precision = precision;
		self
	}

	pub fn build(self) -> Result<NetworkMonitorConfig> {
		self.config.validate()?;
		Ok(self.config)
	}

	pub fn build_unchecked(self) -> NetworkMonitorConfig {
		self.config
	}
}

impl Default for NetworkMonitorConfigBuilder {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum InterfaceFilter {
	ByName(String),
	ByType(u32),
	ByDescription(String),
	Custom(fn(&windows::Win32::NetworkManagement::IpHelper::MIB_IFROW) -> bool),
}

impl InterfaceFilter {
	pub fn matches(&self, interface: &windows::Win32::NetworkManagement::IpHelper::MIB_IFROW) -> bool {
		match self {
			InterfaceFilter::ByName(_name) => false,
			InterfaceFilter::ByType(interface_type) => interface.dwType == *interface_type,
			InterfaceFilter::ByDescription(desc) => unsafe {
				let desc_slice = std::slice::from_raw_parts(interface.bDescr.as_ptr(), interface.dwDescrLen as usize);
				if let Ok(description) = std::str::from_utf8(desc_slice) {
					description.to_lowercase().contains(&desc.to_lowercase())
				} else {
					false
				}
			}
			InterfaceFilter::Custom(f) => f(interface),
		}
	}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PrecisionMode {
	/// Use differential sampling based on previous measurements (default).
	Instant,
	/// Measure over a specific blocking window to improve accuracy.
	Windowed {
		duration: Duration,
	},
	/// Collect multiple instantaneous samples and average them.
	Samples {
		samples: NonZeroU8,
		interval: Duration,
	},
}

impl PrecisionMode {
	pub fn validate(&self) -> Result<()> {
		match self {
			PrecisionMode::Instant => Ok(()),
			PrecisionMode::Windowed { duration } => {
				if duration.is_zero() {
					return Err(NetworkError::InvalidConfiguration {
						field: "precision.windowed.duration must be > 0".to_string(),
					});
				}
				Ok(())
			}
			PrecisionMode::Samples { samples: _, interval } => {
				if interval.is_zero() {
					return Err(NetworkError::InvalidConfiguration {
						field: "precision.samples.interval must be > 0".to_string(),
					});
				}
				Ok(())
			}
		}
	}
}
