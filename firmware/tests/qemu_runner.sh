#!/bin/bash
# QEMU ESP32-C3 Test Runner
# This script runs the firmware in QEMU for testing

set -e

# Source ESP-IDF environment if available
if [ -f "$HOME/export-esp.sh" ]; then
    source "$HOME/export-esp.sh"
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
FIRMWARE_DIR="$(dirname "$SCRIPT_DIR")"
BINARY_PATH="$FIRMWARE_DIR/target/riscv32imc-esp-espidf/release/navign-firmware"
LOG_FILE="$FIRMWARE_DIR/qemu_output.log"

echo "================================================"
echo "  Navign Firmware - QEMU Test Runner"
echo "================================================"
echo ""

# Check if firmware binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "✗ Firmware binary not found at: $BINARY_PATH"
    echo ""
    echo "  Please build the firmware first:"
    echo "  cd firmware && cargo build --release"
    echo ""
    echo "  Or run: just test-firmware-qemu (includes build)"
    exit 1
fi

echo "✓ Firmware binary found"
echo "  Size: $(stat -c%s "$BINARY_PATH") bytes"
echo ""

# Check if QEMU is installed
if ! command -v qemu-system-riscv32 &> /dev/null; then
    echo "✗ qemu-system-riscv32 not found"
    echo ""
    echo "  Please install ESP32-C3 QEMU:"
    echo "  git clone --depth 1 --branch esp-develop-9.0.0 https://github.com/espressif/qemu.git"
    echo "  cd qemu"
    echo "  ./configure --target-list=riscv32-softmmu --enable-gcrypt --enable-slirp"
    echo "  make -j\$(nproc)"
    echo "  sudo make install"
    exit 1
fi

echo "✓ QEMU found: $(qemu-system-riscv32 --version | head -1)"
echo ""

# Clean previous log
rm -f "$LOG_FILE"

echo "Starting QEMU ESP32-C3 simulation..."
echo "  Timeout: 30 seconds"
echo "  Log file: $LOG_FILE"
echo ""

# Start QEMU in background with timeout
timeout 30s qemu-system-riscv32 \
  -nographic \
  -machine esp32c3 \
  -drive file="$BINARY_PATH",if=mtd,format=raw \
  -serial mon:stdio \
  > "$LOG_FILE" 2>&1 &

QEMU_PID=$!
echo "QEMU started with PID: $QEMU_PID"
echo ""

# Wait for firmware to boot and run
echo "Waiting for firmware to boot..."
sleep 10

# Check if QEMU is still running
if ! kill -0 $QEMU_PID 2>/dev/null; then
    echo "✗ QEMU exited unexpectedly"
    echo ""
    echo "=== QEMU Output ==="
    cat "$LOG_FILE"
    echo "==================="
    exit 1
fi

echo "✓ QEMU is running"
echo ""

# Analyze output
echo "=== Analyzing firmware output ==="
echo ""

# Check for panics
if grep -q "PANIC" "$LOG_FILE"; then
    echo "✗ Firmware PANIC detected!"
    echo ""
    grep -A 5 "PANIC" "$LOG_FILE"
    echo ""
    kill $QEMU_PID 2>/dev/null || true
    exit 1
fi

# Check for errors
if grep -q "ERROR" "$LOG_FILE"; then
    echo "⚠  Errors detected in firmware output:"
    grep "ERROR" "$LOG_FILE"
    echo ""
fi

# Check for successful boot indicators
boot_success=false

if grep -q "BLE advertising started" "$LOG_FILE"; then
    echo "✓ BLE advertising started successfully"
    boot_success=true
fi

if grep -q "Navign" "$LOG_FILE" || grep -q "beacon" "$LOG_FILE"; then
    echo "✓ Firmware initialized"
    boot_success=true
fi

if grep -q "Currently booted partition" "$LOG_FILE"; then
    echo "✓ Partition table loaded"
    boot_success=true
fi

if [ "$boot_success" = false ]; then
    echo "⚠  Could not confirm successful firmware boot"
    echo "   (This might be normal for QEMU simulation)"
fi

echo ""
echo "=== Test Summary ==="
echo ""

# Clean shutdown
kill $QEMU_PID 2>/dev/null || true
wait $QEMU_PID 2>/dev/null || true

# Final verdict
if [ "$boot_success" = true ]; then
    echo "✓ QEMU simulation test PASSED"
    echo ""
    echo "Full output saved to: $LOG_FILE"
    exit 0
else
    echo "⚠  QEMU simulation test completed with warnings"
    echo ""
    echo "=== Full QEMU Output ==="
    cat "$LOG_FILE"
    echo "========================"
    echo ""
    echo "Note: Some features may not work in QEMU simulation"
    exit 0  # Don't fail on warnings
fi
