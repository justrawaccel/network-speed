use std::collections::{ HashMap, HashSet };
use windows::{
	Win32::Foundation::{ ERROR_INSUFFICIENT_BUFFER, NO_ERROR },
	Win32::NetworkManagement::IpHelper::{ GetIfTable, MIB_IFTABLE, MIB_IFROW, INTERNAL_IF_OPER_STATUS },
};

use crate::types::{ NetworkError, Result, NetworkMonitorConfig, format_bits_per_second };

#[derive(Debug, Clone)]
pub struct NetworkInterface {
	pub index: u32,
	pub interface_type: u32,
	pub description: String,
	pub is_operational: bool,
	pub bytes_sent: u64,
	pub bytes_received: u64,
	pub speed: u64,
}

impl NetworkInterface {
	pub fn from_mib_ifrow(row: &MIB_IFROW) -> Result<Self> {
		let description = unsafe {
			let desc_slice = std::slice::from_raw_parts(row.bDescr.as_ptr(), row.dwDescrLen as usize);
			String::from_utf8_lossy(desc_slice).into_owned()
		};

		Ok(NetworkInterface {
			index: row.dwIndex,
			interface_type: row.dwType,
			description,
			is_operational: row.dwOperStatus == INTERNAL_IF_OPER_STATUS(1),
			bytes_sent: row.dwOutOctets as u64,
			bytes_received: row.dwInOctets as u64,
			speed: row.dwSpeed as u64,
		})
	}

	pub fn is_virtual(&self) -> bool {
		is_virtual_interface_by_description(&self.description)
	}

	pub fn is_loopback(&self) -> bool {
		self.interface_type == 24
	}

	pub fn is_bluetooth(&self) -> bool {
		self.description.to_lowercase().contains("bluetooth")
	}

	pub fn total_bytes(&self) -> u64 {
		self.bytes_sent.saturating_add(self.bytes_received)
	}

	pub fn type_name(&self) -> &'static str {
		match self.interface_type {
			1 => "Other",
			6 => "Ethernet",
			9 => "Token Ring",
			23 => "PPP",
			24 => "Loopback",
			37 => "Serial",
			71 => "Wi-Fi",
			131 => "Tunnel",
			144 => "WWAN",
			145 => "WiMAX",
			_ => "Unknown",
		}
	}

	pub fn formatted_speed(&self) -> String {
		format_bits_per_second(self.speed)
	}
}

pub struct InterfaceManager {
	config: NetworkMonitorConfig,
	interface_cache: HashMap<u32, NetworkInterface>,
}

impl InterfaceManager {
	pub fn new(config: NetworkMonitorConfig) -> Self {
		Self {
			config,
			interface_cache: HashMap::new(),
		}
	}

	pub fn get_active_interfaces(&mut self) -> Result<Vec<NetworkInterface>> {
		let raw_interfaces = get_raw_interfaces()?;
		let mut active_interfaces = Vec::new();
		let mut active_indices = HashSet::new();

		for raw_interface in raw_interfaces {
			let interface = NetworkInterface::from_mib_ifrow(&raw_interface)?;

			if self.should_include_interface(&interface) {
				self.interface_cache.insert(interface.index, interface.clone());
				active_indices.insert(interface.index);
				active_interfaces.push(interface);
			}
		}

		if active_indices.is_empty() {
			self.interface_cache.clear();
			return Err(NetworkError::NoInterfacesFound);
		}

		self.interface_cache.retain(|index, _| active_indices.contains(index));

		Ok(active_interfaces)
	}

	pub fn get_total_traffic(&mut self) -> Result<(u64, u64)> {
		let interfaces = self.get_active_interfaces()?;

		let total_sent = interfaces
			.iter()
			.map(|i| i.bytes_sent)
			.sum();
		let total_received = interfaces
			.iter()
			.map(|i| i.bytes_received)
			.sum();

		Ok((total_sent, total_received))
	}

	pub fn get_interface_by_index(&self, index: u32) -> Option<&NetworkInterface> {
		self.interface_cache.get(&index)
	}

	pub fn refresh_cache(&mut self) -> Result<()> {
		self.interface_cache.clear();
		self.get_active_interfaces()?;
		Ok(())
	}

	fn should_include_interface(&self, interface: &NetworkInterface) -> bool {
		if self.config.exclude_loopback && interface.is_loopback() {
			return false;
		}

		if self.config.exclude_virtual && interface.is_virtual() {
			return false;
		}

		if self.config.exclude_bluetooth && interface.is_bluetooth() {
			return false;
		}

		if self.config.interface_type_filters.contains(&interface.interface_type) {
			return false;
		}

		if !self.config.interface_name_filters.is_empty() {
			let desc_lower = interface.description.to_lowercase();
			let should_exclude = self.config.interface_name_filters
				.iter()
				.any(|filter| desc_lower.contains(&filter.to_lowercase()));

			if should_exclude {
				return false;
			}
		}

		true
	}
}

fn get_raw_interfaces() -> Result<Vec<MIB_IFROW>> {
	unsafe {
		let mut buffer_size = 0u32;
		let result = GetIfTable(None, &mut buffer_size, false);

		if result != ERROR_INSUFFICIENT_BUFFER.0 {
			return Err(NetworkError::WindowsApi(windows::core::Error::from_win32()));
		}

		let layout = std::alloc::Layout
			::from_size_align(buffer_size as usize, 8)
			.map_err(|_| NetworkError::MemoryAllocation)?;
		let buffer = std::alloc::alloc(layout) as *mut MIB_IFTABLE;

		if buffer.is_null() {
			return Err(NetworkError::MemoryAllocation);
		}

		let result = GetIfTable(Some(buffer), &mut buffer_size, false);
		if result != NO_ERROR.0 {
			std::alloc::dealloc(buffer as *mut u8, layout);
			return Err(NetworkError::WindowsApi(windows::core::Error::from_win32()));
		}

		let table = &*buffer;
		let interfaces = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize).to_vec();

		std::alloc::dealloc(buffer as *mut u8, layout);
		Ok(interfaces)
	}
}

fn is_virtual_interface_by_description(description: &str) -> bool {
	const VIRTUAL_KEYWORDS: &[&str] = &[
		"virtual",
		"vpn",
		"tunnel",
		"tap",
		"tun",
		"vmware",
		"virtualbox",
		"hyper-v",
		"teredo",
		"6to4",
		"microsoft wi-fi direct virtual adapter",
		"isatap",
		"wan miniport",
		"ras async adapter",
		"pptp",
		"l2tp",
		"sstp",
		"ikev2",
		"ppp",
		"dial-up",
	];

	let desc_lower = description.to_lowercase();
	VIRTUAL_KEYWORDS.iter().any(|&keyword| desc_lower.contains(keyword))
}

pub fn list_all_interfaces() -> Result<Vec<NetworkInterface>> {
	let raw_interfaces = get_raw_interfaces()?;
	let mut interfaces = Vec::new();

	for raw_interface in raw_interfaces {
		let interface = NetworkInterface::from_mib_ifrow(&raw_interface)?;
		interfaces.push(interface);
	}

	Ok(interfaces)
}

pub fn get_interface_count() -> Result<usize> {
	let interfaces = get_raw_interfaces()?;
	Ok(interfaces.len())
}
