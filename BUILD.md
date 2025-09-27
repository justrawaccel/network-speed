# Quick Build Instructions

## Building the Rust Library

1. **Build the release version:**
   ```bash
   cargo build --release
   ```

2. **The compiled DLL will be at:**
   ```
   target/release/network_speed.dll
   ```

## Building C++ Examples

1. **Copy DLL to examples directory:**
   ```bash
   copy target\release\network_speed.dll examples\
   cd examples
   ```

2. **Compile with MinGW-w64:**
   ```bash
   g++ -std=c++17 -O2 -o cpp_example.exe cpp_example.cpp -L. -lnetwork_speed
   ```

3. **Or use the automated build script:**
   ```bash
   build_example.bat
   ```

## Quick Test

Run the simple test to verify everything works:

```bash
g++ -o simple_test.exe simple_test.cpp -L. -lnetwork_speed
simple_test.exe
```

## For Windhawk Integration

1. Place `network_speed.dll` in your Windhawk mod directory
2. Use dynamic loading with `LoadLibrary/GetProcAddress`
3. Call `get_net_speed()` every 1-2 seconds for real-time monitoring

## Troubleshooting

- Ensure Visual C++ Redistributable is installed
- DLL must be in same directory as executable or in PATH
- Run with administrator privileges if you get access denied errors
