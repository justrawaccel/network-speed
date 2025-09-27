@echo off
REM Quick test script for network-speed library
REM This script builds the library and runs basic tests

echo Network Speed Library - Quick Test
echo ==================================
echo.

REM Check if Rust is installed
cargo --version >nul 2>&1
if errorlevel 1 (
    echo Error: Rust/Cargo not found! Please install Rust first.
    echo Download from: https://rustup.rs/
    pause
    exit /b 1
)

echo Step 1: Building Rust library...
cargo build --release
if errorlevel 1 (
    echo Error: Failed to build Rust library!
    pause
    exit /b 1
)

echo ✓ Library built successfully

echo.
echo Step 2: Running Rust tests...
cargo test
if errorlevel 1 (
    echo Warning: Some tests failed, but continuing...
) else (
    echo ✓ All Rust tests passed
)

echo.
echo Step 3: Checking DLL...
if exist "target\release\network_speed.dll" (
    echo ✓ DLL found: target\release\network_speed.dll

    REM Get file size
    for %%I in ("target\release\network_speed.dll") do (
        echo   Size: %%~zI bytes
    )
) else (
    echo Error: DLL not found!
    pause
    exit /b 1
)

echo.
echo Step 4: Copying DLL to examples directory...
if not exist "examples" mkdir examples
copy "target\release\network_speed.dll" "examples\" >nul 2>&1
if errorlevel 1 (
    echo Warning: Could not copy DLL to examples directory
) else (
    echo ✓ DLL copied to examples directory
)

echo.
echo Step 5: Checking for C++ compiler...
g++ --version >nul 2>&1
if errorlevel 1 (
    echo Warning: g++ not found, skipping C++ example compilation
    echo You can install MinGW-w64 to compile C++ examples
    goto :skip_cpp
)

echo ✓ Found g++ compiler

echo.
echo Step 6: Building C++ simple test...
cd examples
g++ -std=c++17 -O2 -o simple_test.exe simple_test.cpp -L. -lnetwork_speed 2>nul
if errorlevel 1 (
    echo Warning: Failed to build C++ test (this is normal if missing libraries)
    echo You may need Visual C++ Redistributable installed
) else (
    echo ✓ C++ test built successfully

    echo.
    echo Step 7: Running C++ test...
    echo Running simple_test.exe for 10 seconds...
    timeout /t 2 >nul 2>&1
    simple_test.exe
    if errorlevel 1 (
        echo Warning: C++ test had errors (check output above)
    ) else (
        echo ✓ C++ test completed
    )
)

cd ..

:skip_cpp
echo.
echo ======================
echo Test Summary
echo ======================
echo.
echo ✓ Rust library builds successfully
echo ✓ DLL created: target\release\network_speed.dll
echo ✓ Ready for integration into C++/Windhawk projects
echo.
echo Next steps:
echo 1. Use the DLL in your C++ project
echo 2. For Windhawk: Place DLL in mod directory and use LoadLibrary
echo 3. Call get_net_speed() every 1-2 seconds for real-time monitoring
echo.
echo Example usage:
echo   extern "C" __declspec(dllimport) int get_net_speed(uint64_t* up, uint64_t* down);
echo
echo   uint64_t upload, download;
echo   if (get_net_speed(&upload, &download) == 0) {
echo       printf("Upload: %llu B/s, Download: %llu B/s\n", upload, download);
echo   }
echo.
pause
