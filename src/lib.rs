use std::sync::Mutex;
use std::time::Instant;
use once_cell::sync::Lazy;
use windows::{
    core::Result as WindowsResult,
    Win32::NetworkManagement::IpHelper::{
        GetIfTable, MIB_IFTABLE, MIB_IFROW, INTERNAL_IF_OPER_STATUS,
    },
    Win32::Foundation::{NO_ERROR, ERROR_INSUFFICIENT_BUFFER},
};

/// Network statistics for tracking changes over time
#[derive(Debug, Clone, Default)]
struct NetworkStats {
    total_upload_bytes: u64,
    total_download_bytes: u64,
    timestamp: Option<Instant>,
}

/// Global state to store previous network statistics
static PREVIOUS_STATS: Lazy<Mutex<NetworkStats>> = Lazy::new(|| {
    Mutex::new(NetworkStats::default())
});

/// Get current network interface statistics from Windows IP Helper API
fn get_current_network_stats() -> WindowsResult<NetworkStats> {
    unsafe {
        let mut buffer_size = 0u32;
        let result = GetIfTable(None, &mut buffer_size, false);

        if result != ERROR_INSUFFICIENT_BUFFER.0 {
            return Err(windows::core::Error::from_win32());
        }

        let layout = std::alloc::Layout::from_size_align(buffer_size as usize, 8)
            .map_err(|_| windows::core::Error::from_win32())?;
        let buffer = std::alloc::alloc(layout) as *mut MIB_IFTABLE;
        if buffer.is_null() {
            return Err(windows::core::Error::from_win32());
        }

        let result = GetIfTable(Some(buffer), &mut buffer_size, false);
        if result != NO_ERROR.0 {
            std::alloc::dealloc(buffer as *mut u8, layout);
            return Err(windows::core::Error::from_win32());
        }

        let table = &*buffer;
        let mut total_upload = 0u64;
        let mut total_download = 0u64;

        let interfaces = std::slice::from_raw_parts(
            table.table.as_ptr(),
            table.dwNumEntries as usize
        );

        for interface in interfaces {
            if interface.dwOperStatus == INTERNAL_IF_OPER_STATUS(1) && // IF_OPER_STATUS_UP
               interface.dwType != 24 && // IF_TYPE_SOFTWARE_LOOPBACK
               !is_virtual_interface(&interface) {

                total_upload = total_upload.saturating_add(interface.dwOutOctets as u64);
                total_download = total_download.saturating_add(interface.dwInOctets as u64);
            }
        }

        std::alloc::dealloc(buffer as *mut u8, layout);

        Ok(NetworkStats {
            total_upload_bytes: total_upload,
            total_download_bytes: total_download,
            timestamp: Some(Instant::now()),
        })
    }
}

/// Basic heuristic to identify virtual/tunnel interfaces
fn is_virtual_interface(interface: &MIB_IFROW) -> bool {
    unsafe {
        let desc_slice = std::slice::from_raw_parts(
            interface.bDescr.as_ptr(),
            interface.dwDescrLen as usize
        );

        if let Ok(description) = std::str::from_utf8(desc_slice) {
            let desc_lower = description.to_lowercase();

            return desc_lower.contains("virtual") ||
                   desc_lower.contains("vpn") ||
                   desc_lower.contains("tunnel") ||
                   desc_lower.contains("tap") ||
                   desc_lower.contains("tun") ||
                   desc_lower.contains("vmware") ||
                   desc_lower.contains("virtualbox") ||
                   desc_lower.contains("hyper-v") ||
                   desc_lower.contains("teredo") ||
                   desc_lower.contains("6to4") ||
                   desc_lower.contains("microsoft wi-fi direct virtual adapter") ||
                   desc_lower.contains("bluetooth") ||
                   desc_lower.contains("loopback");
        }
    }

    false
}

/// Calculate network speed based on byte difference and time elapsed
fn calculate_speed(current: &NetworkStats, previous: &NetworkStats) -> (u64, u64) {
    if let (Some(current_time), Some(previous_time)) = (current.timestamp, previous.timestamp) {
        let duration = current_time.duration_since(previous_time);
        let seconds = duration.as_secs_f64();

        if seconds < 0.1 {
            return (0, 0);
        }

        let upload_diff = current.total_upload_bytes.wrapping_sub(previous.total_upload_bytes);
        let download_diff = current.total_download_bytes.wrapping_sub(previous.total_download_bytes);

        if upload_diff > (1u64 << 62) || download_diff > (1u64 << 62) {
            return (0, 0);
        }

        let upload_speed = (upload_diff as f64 / seconds) as u64;
        let download_speed = (download_diff as f64 / seconds) as u64;

        (upload_speed, download_speed)
    } else {
        (0, 0)
    }
}

/// Main exported function for getting network speed
#[no_mangle]
pub extern "C" fn get_net_speed(upload: *mut u64, download: *mut u64) -> i32 {
    let current_stats = match get_current_network_stats() {
        Ok(stats) => stats,
        Err(_) => return -1,
    };

    let mut previous_stats_guard = match PREVIOUS_STATS.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };

    let (upload_speed, download_speed) = if previous_stats_guard.timestamp.is_some() {
        calculate_speed(&current_stats, &*previous_stats_guard)
    } else {
        (0, 0)
    };

    *previous_stats_guard = current_stats;

    unsafe {
        if !upload.is_null() {
            *upload = upload_speed;
        }
        if !download.is_null() {
            *download = download_speed;
        }
    }

    0
}

/// Reset the internal state (useful for testing or reinitializing)
#[no_mangle]
pub extern "C" fn reset_net_speed() -> i32 {
    match PREVIOUS_STATS.lock() {
        Ok(mut guard) => {
            *guard = NetworkStats::default();
            0
        },
        Err(_) => -1,
    }
}

/// Get library version information
#[no_mangle]
pub extern "C" fn get_version() -> *const i8 {
    "1.0.0\0".as_ptr() as *const i8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_network_stats() {
        if cfg!(windows) {
            let stats = get_current_network_stats();
            assert!(stats.is_ok(), "Failed to get network stats: {:?}", stats.err());
        }
    }

    #[test]
    fn test_speed_calculation() {
        let previous = NetworkStats {
            total_upload_bytes: 1000,
            total_download_bytes: 2000,
            timestamp: Some(Instant::now() - std::time::Duration::from_secs(1)),
        };

        let current = NetworkStats {
            total_upload_bytes: 2000,
            total_download_bytes: 4000,
            timestamp: Some(Instant::now()),
        };

        let (upload_speed, download_speed) = calculate_speed(&current, &previous);

        assert!(upload_speed > 500 && upload_speed < 1500, "Upload speed out of range: {}", upload_speed);
        assert!(download_speed > 1500 && download_speed < 2500, "Download speed out of range: {}", download_speed);
    }

    #[test]
    fn test_c_interface() {
        assert_eq!(get_net_speed(std::ptr::null_mut(), std::ptr::null_mut()), 0);

        assert_eq!(reset_net_speed(), 0);

        let mut upload = 0u64;
        let mut download = 0u64;
        assert_eq!(get_net_speed(&mut upload, &mut download), 0);

        assert_eq!(upload, 0);
        assert_eq!(download, 0);
    }
}
