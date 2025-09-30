use std::collections::VecDeque;
use std::time::{ Duration, Instant };
use tokio::time::{ interval, MissedTickBehavior };
use tokio::sync::mpsc;
use std::sync::{ Arc, Mutex };

use crate::types::{ NetworkError, Result, NetworkSpeed, NetworkMonitorConfig };
use crate::monitor::NetworkMonitor;

pub struct AsyncNetworkMonitor {
	inner: Arc<Mutex<NetworkMonitor>>,
}

impl AsyncNetworkMonitor {
	pub fn new() -> Self {
		Self {
			inner: Arc::new(Mutex::new(NetworkMonitor::new())),
		}
	}

	pub fn with_config(config: NetworkMonitorConfig) -> Self {
		Self {
			inner: Arc::new(Mutex::new(NetworkMonitor::with_config(config))),
		}
	}

	pub async fn measure_speed(&self) -> Result<NetworkSpeed> {
		let inner_clone = Arc::clone(&self.inner);
		tokio::task
			::spawn_blocking(move || {
				let mut monitor = inner_clone.lock().map_err(|_| NetworkError::InterfaceOperationFailed {
					reason: "Monitor mutex poisoned".to_string(),
				})?;
				monitor.measure_speed()
			}).await
			.map_err(|_| NetworkError::InterfaceOperationFailed {
				reason: "Task join error".to_string(),
			})?
	}

	pub async fn measure_speed_with_delay(&self, measurement_duration: Duration) -> Result<NetworkSpeed> {
		let inner_clone = Arc::clone(&self.inner);
		let result = tokio::task::spawn_blocking(move || {
			let mut monitor = inner_clone.lock().map_err(|_| NetworkError::InterfaceOperationFailed {
				reason: "Monitor mutex poisoned".to_string(),
			})?;
			monitor.measure_speed_blocking(measurement_duration)
		}).await;

		result.map_err(|_| NetworkError::InterfaceOperationFailed {
			reason: "Task join error".to_string(),
		})?
	}

	pub async fn get_instantaneous_speed(&self) -> Result<Option<NetworkSpeed>> {
		let inner_clone = Arc::clone(&self.inner);
		tokio::task
			::spawn_blocking(move || {
				let mut monitor = inner_clone.lock().map_err(|_| NetworkError::InterfaceOperationFailed {
					reason: "Monitor mutex poisoned".to_string(),
				})?;
				monitor.get_instantaneous_speed()
			}).await
			.map_err(|_| NetworkError::InterfaceOperationFailed {
				reason: "Task join error".to_string(),
			})?
	}

	pub async fn reset(&self) {
		let inner_clone = Arc::clone(&self.inner);
		tokio::task
			::spawn_blocking(move || {
				if let Ok(mut monitor) = inner_clone.lock() {
					monitor.reset();
				}
			}).await
			.ok();
	}

	pub async fn update_config(&self, config: NetworkMonitorConfig) -> Result<()> {
		let inner_clone = Arc::clone(&self.inner);
		tokio::task
			::spawn_blocking(move || {
				let mut monitor = inner_clone.lock().map_err(|_| NetworkError::InterfaceOperationFailed {
					reason: "Monitor mutex poisoned".to_string(),
				})?;
				monitor.update_config(config)
			}).await
			.map_err(|_| NetworkError::InterfaceOperationFailed {
				reason: "Task join error".to_string(),
			})?
	}

	pub async fn get_config(&self) -> NetworkMonitorConfig {
		let inner_clone = Arc::clone(&self.inner);
		match tokio::task::spawn_blocking(move || { inner_clone.lock().map(|monitor| monitor.get_config().clone()) }).await {
			Ok(Ok(config)) => config,
			_ => NetworkMonitorConfig::default(),
		}
	}

	pub async fn monitor_continuously<F>(&self, interval_duration: Duration, mut callback: F) -> Result<()>
		where F: FnMut(Result<NetworkSpeed>) + Send + 'static
	{
		let mut interval_timer = interval(interval_duration);
		interval_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

		loop {
			interval_timer.tick().await;
			let result = self.measure_speed().await;
			callback(result);
		}
	}

	pub async fn monitor_with_channel(
		&self,
		interval_duration: Duration,
		buffer_size: usize
	) -> Result<mpsc::Receiver<Result<NetworkSpeed>>> {
		let (tx, rx) = mpsc::channel(buffer_size);
		let monitor = Arc::clone(&self.inner);

		tokio::spawn(async move {
			let mut interval_timer = interval(interval_duration);
			interval_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

			loop {
				interval_timer.tick().await;

				let result = {
					let monitor_clone = Arc::clone(&monitor);
					let result = tokio::task::spawn_blocking(move || {
						let mut mon = monitor_clone.lock().map_err(|_| NetworkError::InterfaceOperationFailed {
							reason: "Monitor mutex poisoned".to_string(),
						})?;
						mon.measure_speed()
					}).await;

					result
						.map_err(|_| NetworkError::InterfaceOperationFailed {
							reason: "Task join error".to_string(),
						})
						.and_then(|r| r)
				};

				if tx.send(result).await.is_err() {
					break;
				}
			}
		});

		Ok(rx)
	}

	pub async fn collect_samples(&self, sample_count: usize, interval_duration: Duration) -> Result<Vec<NetworkSpeed>> {
		let mut samples = Vec::with_capacity(sample_count);
		let mut interval_timer = interval(interval_duration);
		interval_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

		for _ in 0..sample_count {
			interval_timer.tick().await;
			match self.measure_speed().await {
				Ok(speed) => samples.push(speed),
				Err(NetworkError::InsufficientTimeElapsed { .. }) => {
					continue;
				}
				Err(e) => {
					return Err(e);
				}
			}
		}

		if samples.is_empty() {
			return Err(NetworkError::InsufficientTimeElapsed {
				min_ms: interval_duration.as_millis() as u64,
				actual_ms: 0,
			});
		}

		Ok(samples)
	}

	pub async fn measure_average_speed(
		&self,
		measurement_duration: Duration,
		sample_interval: Duration
	) -> Result<NetworkSpeed> {
		let sample_count = (measurement_duration.as_millis() / sample_interval.as_millis()) as usize;
		if sample_count == 0 {
			return Err(NetworkError::InvalidConfiguration {
				field: "measurement_duration must be greater than sample_interval".to_string(),
			});
		}

		let samples = self.collect_samples(sample_count, sample_interval).await?;

		let avg_upload =
			samples
				.iter()
				.map(|s| s.upload_bytes_per_sec)
				.sum::<u64>() / (samples.len() as u64);

		let avg_download =
			samples
				.iter()
				.map(|s| s.download_bytes_per_sec)
				.sum::<u64>() / (samples.len() as u64);

		Ok(NetworkSpeed::new(avg_upload, avg_download))
	}
}

impl Default for AsyncNetworkMonitor {
	fn default() -> Self {
		Self::new()
	}
}

pub struct AsyncNetworkSpeedTracker {
	monitor: AsyncNetworkMonitor,
	history: Arc<Mutex<VecDeque<NetworkSpeed>>>,
	max_history_size: usize,
}

impl AsyncNetworkSpeedTracker {
	pub fn new(max_history_size: usize) -> Self {
		Self {
			monitor: AsyncNetworkMonitor::new(),
			history: Arc::new(Mutex::new(VecDeque::with_capacity(max_history_size))),
			max_history_size,
		}
	}

	pub fn with_config(config: NetworkMonitorConfig, max_history_size: usize) -> Self {
		Self {
			monitor: AsyncNetworkMonitor::with_config(config),
			history: Arc::new(Mutex::new(VecDeque::with_capacity(max_history_size))),
			max_history_size,
		}
	}

	pub async fn track_speed(&self) -> Result<NetworkSpeed> {
		let speed = self.monitor.measure_speed().await?;

		let history_clone = Arc::clone(&self.history);
		let max_size = self.max_history_size;
		let speed_clone = speed.clone();

		tokio::task
			::spawn_blocking(
				move || -> Result<(), NetworkError> {
					let mut history = history_clone.lock().map_err(|_| NetworkError::InterfaceOperationFailed {
						reason: "History mutex poisoned".to_string(),
					})?;
					history.push_back(speed_clone);

					if history.len() > max_size {
						history.pop_front();
					}

					Ok(())
				}
			).await
			.map_err(|_| NetworkError::InterfaceOperationFailed {
				reason: "Task join error".to_string(),
			})??;

		Ok(speed)
	}

	pub async fn get_history(&self) -> Vec<NetworkSpeed> {
		let history_clone = Arc::clone(&self.history);
		match
			tokio::task::spawn_blocking(move || {
				let history = history_clone.lock().map_err(|_| NetworkError::InterfaceOperationFailed {
					reason: "History mutex poisoned".to_string(),
				})?;
				Ok(history.iter().cloned().collect::<Vec<_>>())
			}).await
		{
			Ok(Ok(history)) => history,
			_ => Vec::new(),
		}
	}

	pub async fn get_average_speed(&self, duration: Duration) -> Option<NetworkSpeed> {
		let history_clone = Arc::clone(&self.history);
		tokio::task
			::spawn_blocking(move || {
				let history = history_clone.lock().map_err(|_| NetworkError::InterfaceOperationFailed {
					reason: "History mutex poisoned".to_string(),
				})?;

				if history.is_empty() {
					return Ok(None);
				}

				let cutoff_time = Instant::now() - duration;
				let recent_speeds: Vec<_> = history
					.iter()
					.filter(|speed| speed.timestamp >= cutoff_time)
					.collect();

				if recent_speeds.is_empty() {
					return Ok(None);
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

				Ok(Some(NetworkSpeed::new(avg_upload, avg_download)))
			}).await
			.ok()
			.and_then(|result| result.ok())
			.flatten()
	}

	pub async fn get_peak_speed(&self, duration: Duration) -> Option<NetworkSpeed> {
		let history_clone = Arc::clone(&self.history);
		tokio::task
			::spawn_blocking(move || {
				let history = history_clone.lock().map_err(|_| NetworkError::InterfaceOperationFailed {
					reason: "History mutex poisoned".to_string(),
				})?;

				if history.is_empty() {
					return Ok(None);
				}

				let cutoff_time = Instant::now() - duration;
				let peak = history
					.iter()
					.filter(|speed| speed.timestamp >= cutoff_time)
					.max_by_key(|speed| speed.total_bytes_per_sec())
					.cloned();

				Ok(peak)
			}).await
			.ok()
			.and_then(|result| result.ok())
			.flatten()
	}

	pub async fn clear_history(&self) {
		let history_clone = Arc::clone(&self.history);
		tokio::task
			::spawn_blocking(move || {
				if let Ok(mut history) = history_clone.lock() {
					history.clear();
				}
			}).await
			.ok();
	}

	pub async fn reset(&self) {
		self.monitor.reset().await;
		self.clear_history().await;
	}

	pub async fn start_continuous_tracking(
		&self,
		interval_duration: Duration
	) -> Result<mpsc::Receiver<Result<NetworkSpeed>>> {
		let (tx, rx) = mpsc::channel(100);
		let history_clone = Arc::clone(&self.history);
		let max_size = self.max_history_size;
		let monitor = AsyncNetworkMonitor::with_config(self.monitor.get_config().await);

		tokio::spawn(async move {
			let mut interval_timer = interval(interval_duration);
			interval_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

			loop {
				interval_timer.tick().await;

				let result = monitor.measure_speed().await;

				if let Ok(ref speed) = result {
					let hist_clone = Arc::clone(&history_clone);
					let speed_clone = speed.clone();
					tokio::task
						::spawn_blocking(move || {
							if let Ok(mut hist) = hist_clone.lock() {
								hist.push_back(speed_clone);
								if hist.len() > max_size {
									hist.pop_front();
								}
							}
						}).await
						.ok();
				}

				if tx.send(result).await.is_err() {
					break;
				}
			}
		});

		Ok(rx)
	}
}
