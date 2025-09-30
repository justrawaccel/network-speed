use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
	#[error("Windows API error: {0}")] WindowsApi(#[from] windows::core::Error),

	#[error("Memory allocation failed")]
	MemoryAllocation,

	#[error("Invalid interface data")]
	InvalidInterface,

	#[error(
		"Insufficient time elapsed for accurate measurement (minimum: {min_ms}ms, actual: {actual_ms}ms)"
	)] InsufficientTimeElapsed {
		min_ms: u64,
		actual_ms: u64,
	},

	#[error("No network interfaces found")]
	NoInterfacesFound,

	#[error("Interface operation failed: {reason}")] InterfaceOperationFailed {
		reason: String,
	},

	#[error("Calculation overflow detected")]
	CalculationOverflow,

	#[error("Invalid configuration: {field}")] InvalidConfiguration {
		field: String,
	},
}

pub type Result<T> = std::result::Result<T, NetworkError>;

impl NetworkError {
	pub fn is_recoverable(&self) -> bool {
		matches!(self, NetworkError::InsufficientTimeElapsed { .. } | NetworkError::CalculationOverflow)
	}

	pub fn error_code(&self) -> u32 {
		match self {
			NetworkError::WindowsApi(_) => 1001,
			NetworkError::MemoryAllocation => 1002,
			NetworkError::InvalidInterface => 1003,
			NetworkError::InsufficientTimeElapsed { .. } => 1004,
			NetworkError::NoInterfacesFound => 1005,
			NetworkError::InterfaceOperationFailed { .. } => 1006,
			NetworkError::CalculationOverflow => 1007,
			NetworkError::InvalidConfiguration { .. } => 1008,
		}
	}
}
