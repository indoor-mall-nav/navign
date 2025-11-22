"""Tests for ESP tool wrappers."""

import shutil
from pathlib import Path

import pytest

from navign_maintenance import esp_tools


def test_check_espefuse_available():
    """Test checking if espefuse is available."""
    # This test will succeed if espefuse.py is installed, otherwise it's expected to fail
    # We just test that the function doesn't crash
    result = esp_tools.check_espefuse_available()
    assert isinstance(result, bool)


def test_detect_flash_tool_returns_string_or_raises():
    """Test that detect_flash_tool either returns a string or raises ESPToolNotFoundError."""
    try:
        tool = esp_tools.detect_flash_tool()
        assert tool in ["espflash", "esptool.py"]
    except esp_tools.ESPToolNotFoundError:
        # Expected if no tool is installed
        pass


def test_generate_device_id_format():
    """Test device ID format (moved from crypto test but kept for compatibility)."""
    from navign_maintenance.crypto import generate_device_id

    device_id = generate_device_id()
    assert len(device_id) == 24
    assert all(c in "0123456789abcdef" for c in device_id)


@pytest.mark.integration
def test_get_chip_info_requires_hardware():
    """Test getting chip info (requires actual hardware)."""
    # This test is marked as integration and will be skipped in normal runs
    # Run with: pytest -m integration
    pytest.skip("Requires ESP32-C3 hardware connected")


@pytest.mark.integration
def test_fuse_key_requires_hardware(temp_keys_dir):
    """Test eFuse programming (requires actual hardware)."""
    pytest.skip("Requires ESP32-C3 hardware connected")
