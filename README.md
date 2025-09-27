# Network Speed Monitor Library

A high-performance Rust library that provides C-compatible interface for monitoring network interface statistics on Windows using the IP Helper API.

## Features

- **Real-time monitoring**: Track upload and download speeds across all active network interfaces
- **Windows Integration**: Uses native Windows IP Helper API (`iphlpapi.dll`) for accurate statistics
- **Smart Filtering**: Automatically excludes loopback, virtual, and inactive interfaces
- **C-Compatible**: Exports C-style functions for easy integration with other languages
- **Thread-Safe**: Uses Mutex for safe concurrent access to internal state
- **Memory Safe**: Built with Rust for guaranteed memory safety

## Quick Start

### Build the Library

```bash
cargo build --release
```

The compiled DLL will be at `target/release/network_speed.dll`

### C++ Usage

```cpp
#include <iostream>

extern "C" {
    __declspec(dllimport) int get_net_speed(uint64_t* upload, uint64_t* download);
    __declspec(dllimport) int reset_net_speed();
}

int main() {
    uint64_t upload = 0, download = 0;

    reset_net_speed();  // Initialize

    if (get_net_speed(&upload, &download) == 0) {
        std::cout << "Upload: " << upload << " B/s, Download: " << download << " B/s" << std::endl;
    }

    return 0;
}
```

### API Reference

#### `get_net_speed(upload, download)`

- **Returns**: `0` on success, `-1` on error
- **Parameters**: Pointers to store speeds in bytes/second (can be NULL)
- **Note**: First call returns 0 speeds (no previous data to compare)

#### `reset_net_speed()`

- **Returns**: `0` on success, `-1` on error
- **Purpose**: Resets internal state for clean measurements

## Integration Examples

### Windhawk Mod

```cpp
// Dynamic loading
HMODULE dll = LoadLibrary(L"network_speed.dll");
auto get_net_speed = (int(*)(uint64_t*, uint64_t*))GetProcAddress(dll, "get_net_speed");

// Use in timer callback every 1-2 seconds
uint64_t up, down;
if (get_net_speed && get_net_speed(&up, &down) == 0) {
    UpdateNetworkDisplay(up, down);
}
```

### Python

```python
import ctypes

dll = ctypes.CDLL('./network_speed.dll')
dll.get_net_speed.argtypes = [ctypes.POINTER(ctypes.c_uint64), ctypes.POINTER(ctypes.c_uint64)]

upload = ctypes.c_uint64()
download = ctypes.c_uint64()
if dll.get_net_speed(ctypes.byref(upload), ctypes.byref(download)) == 0:
    print(f"Upload: {upload.value} B/s, Download: {download.value} B/s")
```

## How It Works

1. **Interface Discovery**: Uses `GetIfTable` to enumerate network interfaces
2. **Filtering**: Excludes loopback, virtual adapters, and inactive interfaces
3. **Data Collection**: Sums `dwInOctets` and `dwOutOctets` from active interfaces
4. **Speed Calculation**: Compares current values with previous measurements over time
5. **Thread Safety**: Global state protected with Mutex for concurrent access

## Building Examples

```bash
# Build C++ example
cd examples
copy ..\target\release\network_speed.dll .
g++ -o example.exe cpp_example.cpp -L. -lnetwork_speed

# Or use automated script
build_example.bat
```

## Requirements

- Windows 10/11 (Windows 7+ should work)
- Rust toolchain for building
- Visual C++ Redistributable for running

## Interface Filtering

**Included**: Physical Ethernet, Wi-Fi, USB-to-Ethernet adapters
**Excluded**: Loopback, VPN, virtual machine adapters, tunnel interfaces

## Performance

- **CPU Usage**: Minimal (~1-5ms per call)
- **Memory**: ~1KB for interface caching
- **Accuracy**: Reflects actual OS network counters
- **Thread Safety**: Safe for concurrent access

## License

[Your license here]
