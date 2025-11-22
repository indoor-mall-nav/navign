"""
ESP32-C3 hardware interaction tools.

Wrappers for espefuse.py (eFuse programming) and firmware flashing tools.
"""

import shutil
import subprocess
from pathlib import Path
from typing import Optional


class ESPToolNotFoundError(Exception):
    """Raised when required ESP tool is not found."""

    pass


def check_espefuse_available() -> bool:
    """
    Check if espefuse.py is available on the system.

    Returns:
        True if espefuse.py is found, False otherwise
    """
    # Try direct command
    if shutil.which("espefuse.py"):
        return True

    # Try python -m espefuse
    for python_cmd in ["python3", "python"]:
        if shutil.which(python_cmd):
            try:
                result = subprocess.run(
                    [python_cmd, "-m", "espefuse", "--help"],
                    capture_output=True,
                    timeout=5,
                )
                if result.returncode == 0:
                    return True
            except (subprocess.TimeoutExpired, FileNotFoundError):
                pass

    return False


def get_chip_info(port: Optional[str] = None) -> str:
    """
    Get ESP32-C3 chip information using espefuse.py.

    Args:
        port: Serial port (e.g., /dev/ttyUSB0)

    Returns:
        Chip information string

    Raises:
        ESPToolNotFoundError: If espefuse.py is not available
        subprocess.CalledProcessError: If espefuse.py fails
    """
    if not check_espefuse_available():
        raise ESPToolNotFoundError(
            "espefuse.py not found. Install with: pip install esptool"
        )

    cmd = ["espefuse.py", "--chip", "esp32c3"]
    if port:
        cmd.extend(["--port", port])
    cmd.append("summary")

    result = subprocess.run(cmd, capture_output=True, text=True, check=True)

    # Extract relevant chip info from output
    for line in result.stdout.splitlines():
        if "Chip is" in line or "Features:" in line:
            return line.strip()

    return "ESP32-C3 detected"


def fuse_key_to_efuse(
    key_file: Path, port: Optional[str] = None, force: bool = False
) -> None:
    """
    Burn private key to ESP32-C3 eFuse BLOCK_KEY0.

    Args:
        key_file: Path to 32-byte private key file
        port: Serial port (e.g., /dev/ttyUSB0)
        force: Skip manual confirmation (adds --do-not-confirm flag)

    Raises:
        ESPToolNotFoundError: If espefuse.py is not available
        subprocess.CalledProcessError: If fusing fails
    """
    if not check_espefuse_available():
        raise ESPToolNotFoundError(
            "espefuse.py not found. Install with: pip install esptool"
        )

    if not key_file.exists():
        raise FileNotFoundError(f"Key file not found: {key_file}")

    cmd = ["espefuse.py", "--chip", "esp32c3"]
    if port:
        cmd.extend(["--port", port])
    
    # Add --do-not-confirm if force is True to skip "BURN" confirmation
    if force:
        cmd.append("--do-not-confirm")
    
    cmd.extend(["burn_key", "BLOCK_KEY0", str(key_file), "USER"])

    print(f"   Executing: {' '.join(cmd)}")

    result = subprocess.run(cmd, capture_output=True, text=True, check=True)

    # Print output
    print("   eFuse programming output:")
    for line in result.stdout.splitlines():
        if line.strip():
            print(f"   {line}")


def detect_flash_tool() -> str:
    """
    Detect available firmware flashing tool.

    Returns:
        Name of detected tool ("espflash" or "esptool.py")

    Raises:
        ESPToolNotFoundError: If no flash tool is found
    """
    # Try espflash first (Rust-based, faster)
    if shutil.which("espflash"):
        return "espflash"

    # Try esptool.py
    if shutil.which("esptool.py"):
        return "esptool.py"

    # Try python -m esptool
    for python_cmd in ["python3", "python"]:
        if shutil.which(python_cmd):
            try:
                result = subprocess.run(
                    [python_cmd, "-m", "esptool", "version"],
                    capture_output=True,
                    timeout=5,
                )
                if result.returncode == 0:
                    return "esptool.py"
            except (subprocess.TimeoutExpired, FileNotFoundError):
                pass

    raise ESPToolNotFoundError(
        "No flash tool found. Install:\n"
        "  - espflash: cargo install espflash (recommended)\n"
        "  - esptool: pip install esptool"
    )


def flash_with_espflash(
    firmware_path: Path,
    port: str,
    baud: int = 921600,
    erase: bool = False,
    verify: bool = True,
    monitor: bool = False,
) -> None:
    """
    Flash firmware using espflash.

    Args:
        firmware_path: Path to firmware binary
        port: Serial port
        baud: Baud rate
        erase: Erase flash before flashing
        verify: Verify flash after writing
        monitor: Monitor serial output after flashing
    """
    cmd = ["espflash", "flash", "--port", port, "--baud", str(baud)]

    if erase:
        cmd.extend(["--erase-parts", "all"])

    if not verify:
        cmd.append("--no-verify")

    if monitor:
        cmd.append("--monitor")

    cmd.append(str(firmware_path))

    print(f"   Executing: {' '.join(cmd)}")

    subprocess.run(cmd, check=True)


def flash_with_esptool(
    firmware_path: Path,
    port: str,
    baud: int = 921600,
    erase: bool = False,
    verify: bool = True,
    monitor: bool = False,
) -> None:
    """
    Flash firmware using esptool.py.

    Args:
        firmware_path: Path to firmware binary
        port: Serial port
        baud: Baud rate
        erase: Erase flash before flashing
        verify: Verify flash (always true for esptool)
        monitor: Monitor serial output (handled separately)
    """
    # Erase flash if requested
    if erase:
        print("   Erasing flash...")
        erase_cmd = ["esptool.py", "--chip", "esp32c3", "--port", port, "erase_flash"]
        subprocess.run(erase_cmd, check=True)
        print("   ✅ Flash erased")

    # Flash firmware
    cmd = [
        "esptool.py",
        "--chip",
        "esp32c3",
        "--port",
        port,
        "--baud",
        str(baud),
        "write_flash",
        "0x0",
        str(firmware_path),
    ]

    print(f"   Executing: {' '.join(cmd)}")

    subprocess.run(cmd, check=True)


def monitor_serial(port: str, baud: int = 115200) -> None:
    """
    Monitor serial output using available tool.

    Args:
        port: Serial port
        baud: Baud rate
    """
    # Try screen
    if shutil.which("screen"):
        cmd = ["screen", port, str(baud)]
        subprocess.run(cmd)
        return

    # Try minicom
    if shutil.which("minicom"):
        cmd = ["minicom", "-D", port, "-b", str(baud)]
        subprocess.run(cmd)
        return

    print("⚠️  No serial monitor tool found (screen or minicom)")
    print("   Install with: sudo apt-get install screen")
