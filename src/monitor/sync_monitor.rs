use std::collections::VecDeque;
use std::time::{ Duration, Instant };

use crate::monitor::InterfaceManager;
use crate::types::{ InterfaceStats, NetworkError, NetworkMonitorConfig, NetworkSpeed, PrecisionMode, Result };

pub struct NetworkMonitor {
	config: NetworkMonitorConfig,
	interface_manager: InterfaceManager,
	previous_stats: Option<InterfaceStats>,
}

impl NetworkMonitor {
	pub fn new() -> Self {
		Self::with_config(NetworkMonitorConfig::default())
	}

	pub fn with_config(config: NetworkMonitorConfig) -> Self {
		let interface_manager = InterfaceManager::new(config.clone());

		Self {
			config,
			interface_manager,
			previous_stats: None,
		}
	}

	pub fn measure_speed(&mut self) -> Result<NetworkSpeed> {
		match &self.config.precision {
			PrecisionMode::Instant => self.measure_instant(),
			PrecisionMode::Windowed { duration } => self.measure_windowed(*duration),
			PrecisionMode::Samples { samples, interval } => { self.measure_samples(samples.get(), *interval) }
		}
	}

	pub fn measure_speed_blocking(&mut self, measurement_duration: Duration) -> Result<NetworkSpeed> {
		self.measure_windowed(measurement_duration)
	}

	pub fn get_instantaneous_speed(&mut self) -> Result<Option<NetworkSpeed>> {
		if self.previous_stats.is_none() {
			return Ok(None);
		}

		match self.measure_speed() {
			Ok(speed) => Ok(Some(speed)),
			Err(NetworkError::InsufficientTimeElapsed { .. }) => Ok(None),
			Err(e) => Err(e),
		}
	}

	pub fn reset(&mut self) {
		self.previous_stats = None;
	}

	pub fn refresh_interfaces(&mut self) -> Result<()> {
		self.interface_manager.refresh_cache()
	}

	pub fn get_config(&self) -> &NetworkMonitorConfig {
		&self.config
	}

	pub fn update_config(&mut self, config: NetworkMonitorConfig) -> Result<()> {
		config.validate()?;
		self.config = config.clone();
		self.interface_manager = InterfaceManager::new(config);
		self.reset();
		Ok(())
	}

	fn get_current_stats(&mut self) -> Result<InterfaceStats> {
		let (total_sent, total_received) = self.interface_manager.get_total_traffic()?;

		Ok(InterfaceStats {
			bytes_sent: total_sent,
			bytes_received: total_received,
			last_update: Instant::now(),
		})
	}

	fn measure_instant(&mut self) -> Result<NetworkSpeed> {
		let current_stats = self.get_current_stats()?;
		let timestamp = current_stats.last_update;

		let speed = if let Some(ref previous) = self.previous_stats {
			self.calculate_speed(&current_stats, previous, timestamp)?
		} else {
			NetworkSpeed::new(0, 0)
		};

		self.previous_stats = Some(current_stats);
		Ok(speed)
	}

	fn measure_windowed(&mut self, duration: Duration) -> Result<NetworkSpeed> {
		let initial_stats = self.get_current_stats()?;
		std::thread::sleep(duration);
		let final_stats = self.get_current_stats()?;
		let timestamp = final_stats.last_update;
		let speed = self.calculate_speed(&final_stats, &initial_stats, timestamp)?;
		self.previous_stats = Some(final_stats);
		Ok(speed)
	}

	fn measure_samples(&mut self, samples: u8, interval: Duration) -> Result<NetworkSpeed> {
		let mut total_upload: u128 = 0;
		let mut total_download: u128 = 0;

		for _ in 0..samples {
			let speed = self.measure_windowed(interval)?;
			total_upload += speed.upload_bytes_per_sec as u128;
			total_download += speed.download_bytes_per_sec as u128;
		}

		let count = samples as u128;
		let avg_upload = (total_upload / count) as u64;
		let avg_download = (total_download / count) as u64;

		Ok(NetworkSpeed::new(avg_upload, avg_download))
	}

	fn calculate_speed(
		&self,
		current: &InterfaceStats,
		previous: &InterfaceStats,
		timestamp: Instant
	) -> Result<NetworkSpeed> {
		let duration = timestamp.duration_since(previous.last_update);

		if duration < self.config.min_measurement_interval {
			return Err(NetworkError::InsufficientTimeElapsed {
				min_ms: self.config.min_measurement_interval.as_millis() as u64,
				actual_ms: duration.as_millis() as u64,
			});
		}

		let seconds = duration.as_secs_f64();
		if seconds <= 0.0 {
			return Err(NetworkError::InsufficientTimeElapsed {
				min_ms: self.config.min_measurement_interval.as_millis() as u64,
				actual_ms: 0,
			});
		}

		let upload_diff = current.bytes_sent.wrapping_sub(previous.bytes_sent);
		let download_diff = current.bytes_received.wrapping_sub(previous.bytes_received);

		if upload_diff > self.config.max_counter_wrap_threshold || download_diff > self.config.max_counter_wrap_threshold {
			return Err(NetworkError::CalculationOverflow);
		}

		let upload_speed = ((upload_diff as f64) / seconds) as u64;
		let download_speed = ((download_diff as f64) / seconds) as u64;

		Ok(NetworkSpeed {
			upload_bytes_per_sec: upload_speed,
			download_bytes_per_sec: download_speed,
			timestamp,
		})
	}
}

impl Default for NetworkMonitor {
	fn default() -> Self {
		Self::new()
	}
}

pub struct NetworkSpeedTracker {
	monitor: NetworkMonitor,
	history: VecDeque<NetworkSpeed>,
	max_history_size: usize,
}

impl NetworkSpeedTracker {
	pub fn new(max_history_size: usize) -> Self {
		Self {
			monitor: NetworkMonitor::new(),
			history: VecDeque::with_capacity(max_history_size),
			max_history_size,
		}
	}

	pub fn with_config(config: NetworkMonitorConfig, max_history_size: usize) -> Self {
		Self {
			monitor: NetworkMonitor::with_config(config),
			history: VecDeque::with_capacity(max_history_size),
			max_history_size,
		}
	}

	pub fn track_speed(&mut self) -> Result<NetworkSpeed> {
		let speed = self.monitor.measure_speed()?;

		self.history.push_back(speed.clone());

		if self.history.len() > self.max_history_size {
			self.history.pop_front();
		}

		Ok(speed)
	}

	pub fn get_history(&self) -> Vec<NetworkSpeed> {
		self.history.iter().cloned().collect()
	}

	pub fn get_average_speed(&self, duration: Duration) -> Option<NetworkSpeed> {
		if self.history.is_empty() {
			return None;
		}

		let cutoff_time = Instant::now() - duration;
		let recent_speeds: Vec<_> = self.history
			.iter()
			.filter(|speed| speed.timestamp >= cutoff_time)
			.collect();

		if recent_speeds.is_empty() {
			return None;
		}

		let avg_upload =
			recent_speeds
				.iter()
				.map(|s| s.upload_bytes_per_sec)
				.sum::<u64>() / (recent_speeds.len() as u64);

		let avg_download =
			recent_speeds
				.iter()
				.map(|s| s.download_bytes_per_sec)
				.sum::<u64>() / (recent_speeds.len() as u64);

		Some(NetworkSpeed::new(avg_upload, avg_download))
	}

	pub fn get_peak_speed(&self, duration: Duration) -> Option<NetworkSpeed> {
		if self.history.is_empty() {
			return None;
		}

		let cutoff_time = Instant::now() - duration;
		self.history
			.iter()
			.filter(|speed| speed.timestamp >= cutoff_time)
			.max_by_key(|speed| speed.total_bytes_per_sec())
			.cloned()
	}

	pub fn clear_history(&mut self) {
		self.history.clear();
	}

	pub fn reset(&mut self) {
		self.monitor.reset();
		self.clear_history();
	}
}
