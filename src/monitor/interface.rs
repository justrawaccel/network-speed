use std::collections::{ HashMap, HashSet };
use windows::{
	core::HRESULT,
	Win32::Foundation::{ ERROR_INSUFFICIENT_BUFFER, ERROR_INVALID_FUNCTION, FALSE, NO_ERROR },
	Win32::NetworkManagement::IpHelper::{
		FreeMibTable,
		GetIfTable,
		GetIfTable2,
		MIB_IFROW,
		MIB_IFTABLE,
		MIB_IF_ROW2,
		MIB_IF_TABLE2,
	},
};

use crate::types::{ format_bits_per_second, NetworkError, NetworkMonitorConfig, Result };

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
	pub fn from_mib_ifrow(row: &MIB_IF_ROW2) -> Result<Self> {
		let description = utf16_to_string(&row.Description);
		let alias = utf16_to_string(&row.Alias);
		let friendly = if !alias.is_empty() { alias } else { description.clone() };

		let transmit_speed = if row.TransmitLinkSpeed == 0 { row.ReceiveLinkSpeed } else { row.TransmitLinkSpeed };

		Ok(NetworkInterface {
			index: row.InterfaceIndex,
			interface_type: row.Type,
			description: if friendly.is_empty() {
				description
			} else {
				friendly
			},
			// NET_IF_OPER_STATUS_UP is defined as 1.
			is_operational: row.OperStatus.0 == 1,
			bytes_sent: row.OutOctets,
			bytes_received: row.InOctets,
			speed: transmit_speed,
		})
	}

	pub fn from_legacy_mib_ifrow(row: &MIB_IFROW) -> Result<Self> {
		let desc_len = (row.dwDescrLen as usize).min(row.bDescr.len());
		let description = String::from_utf8_lossy(&row.bDescr[..desc_len])
			.trim()
			.to_string();
		let friendly = description.clone();

		Ok(NetworkInterface {
			index: row.dwIndex,
			interface_type: row.dwType,
			description: if friendly.is_empty() {
				description
			} else {
				friendly
			},
			is_operational: row.dwOperStatus.0 == 1,
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
		let enumerated = get_raw_interfaces()?;
		let mut active_interfaces = Vec::new();
		let mut active_indices = HashSet::new();

		for interface in enumerated {
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
		if
			!self.config.include_interface_indices.is_empty() &&
			!self.config.include_interface_indices.contains(&interface.index)
		{
			return false;
		}

		let desc_lower = interface.description.to_lowercase();

		if
			!self.config.include_interface_name_patterns.is_empty() &&
			!self.config.include_interface_name_patterns.iter().any(|pattern| desc_lower.contains(&pattern.to_lowercase()))
		{
			return false;
		}

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

fn get_raw_interfaces() -> Result<Vec<NetworkInterface>> {
	let result = unsafe { collect_interfaces_v2() };

	match result {
		Ok(interfaces) => Ok(interfaces),
		Err(NetworkError::WindowsApi(err)) if err.code() == HRESULT::from_win32(ERROR_INVALID_FUNCTION.0 as u32) => unsafe {
			collect_interfaces_v1()
		}
		Err(e) => Err(e),
	}
}

unsafe fn collect_interfaces_v2() -> Result<Vec<NetworkInterface>> {
	let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
	GetIfTable2(&mut table_ptr).map_err(NetworkError::WindowsApi)?;

	let table = &*table_ptr;
	let slice = std::slice::from_raw_parts(table.Table.as_ptr(), table.NumEntries as usize);
	let mut interfaces = Vec::with_capacity(slice.len());

	for row in slice {
		interfaces.push(NetworkInterface::from_mib_ifrow(row)?);
	}

	if let Err(err) = FreeMibTable(table_ptr as _) {
		return Err(NetworkError::WindowsApi(err));
	}

	Ok(interfaces)
}

unsafe fn collect_interfaces_v1() -> Result<Vec<NetworkInterface>> {
	let mut size = 0u32;
	let mut status = GetIfTable(None, &mut size, FALSE);
	if status != ERROR_INSUFFICIENT_BUFFER.0 {
		let err = windows::core::Error::from(HRESULT::from_win32(status));
		return Err(NetworkError::WindowsApi(err));
	}

	let mut buffer = vec![0u8; size as usize];
	let table_ptr = buffer.as_mut_ptr() as *mut MIB_IFTABLE;
	status = GetIfTable(Some(table_ptr), &mut size, FALSE);
	if status != NO_ERROR.0 {
		let err = windows::core::Error::from(HRESULT::from_win32(status));
		return Err(NetworkError::WindowsApi(err));
	}

	let table = &*table_ptr;
	let rows = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);
	let mut interfaces = Vec::with_capacity(rows.len());

	for row in rows {
		interfaces.push(NetworkInterface::from_legacy_mib_ifrow(row)?);
	}

	Ok(interfaces)
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
	get_raw_interfaces()
}

pub fn get_interface_count() -> Result<usize> {
	Ok(get_raw_interfaces()?.len())
}

fn utf16_to_string(buf: &[u16]) -> String {
	let len = buf
		.iter()
		.position(|&c| c == 0)
		.unwrap_or(buf.len());
	String::from_utf16_lossy(&buf[..len])
		.trim()
		.to_string()
}
