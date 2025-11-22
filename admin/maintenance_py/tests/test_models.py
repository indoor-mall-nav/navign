"""Tests for key metadata models."""

import json
from pathlib import Path

import pytest

from navign_maintenance.models import KeyMetadata


def test_key_metadata_creation():
    """Test creating KeyMetadata."""
    metadata = KeyMetadata(
        key_name="test_key",
        private_key_file="test_key_private.bin",
        public_key_hex="04" + "a1" * 32 + "b2" * 32,
        generated_at="2025-01-15T10:30:00Z",
        fused=False,
        chip_info=None,
    )

    assert metadata.key_name == "test_key"
    assert metadata.private_key_file == "test_key_private.bin"
    assert len(metadata.public_key_hex) == 130
    assert metadata.fused is False
    assert metadata.chip_info is None


def test_key_metadata_to_json():
    """Test metadata JSON serialization."""
    metadata = KeyMetadata(
        key_name="test_key",
        private_key_file="test_key_private.bin",
        public_key_hex="04aabbcc",
        generated_at="2025-01-15T10:30:00Z",
        fused=False,
        chip_info=None,
    )

    json_str = metadata.to_json()

    # Verify it's valid JSON
    data = json.loads(json_str)
    assert data["key_name"] == "test_key"
    assert data["fused"] is False


def test_key_metadata_from_json():
    """Test metadata JSON deserialization."""
    json_str = """
    {
        "key_name": "test_key",
        "private_key_file": "test_key_private.bin",
        "public_key_hex": "04aabbcc",
        "generated_at": "2025-01-15T10:30:00Z",
        "fused": true,
        "chip_info": "ESP32-C3 detected"
    }
    """

    metadata = KeyMetadata.from_json(json_str)

    assert metadata.key_name == "test_key"
    assert metadata.fused is True
    assert metadata.chip_info == "ESP32-C3 detected"


def test_key_metadata_save_and_load(temp_keys_dir):
    """Test saving and loading metadata from file."""
    metadata = KeyMetadata(
        key_name="test_key",
        private_key_file="test_key_private.bin",
        public_key_hex="04aabbcc",
        generated_at="2025-01-15T10:30:00Z",
        fused=False,
        chip_info=None,
    )

    # Save to file
    metadata_path = temp_keys_dir / "metadata.json"
    metadata.save(metadata_path)

    # Verify file exists
    assert metadata_path.exists()

    # Load from file
    loaded = KeyMetadata.from_file(metadata_path)

    assert loaded.key_name == metadata.key_name
    assert loaded.private_key_file == metadata.private_key_file
    assert loaded.public_key_hex == metadata.public_key_hex
    assert loaded.fused == metadata.fused


def test_key_metadata_update_fused_status(temp_keys_dir):
    """Test updating metadata fused status."""
    metadata = KeyMetadata(
        key_name="test_key",
        private_key_file="test_key_private.bin",
        public_key_hex="04aabbcc",
        generated_at="2025-01-15T10:30:00Z",
        fused=False,
        chip_info=None,
    )

    metadata_path = temp_keys_dir / "metadata.json"
    metadata.save(metadata_path)

    # Load and update
    loaded = KeyMetadata.from_file(metadata_path)
    assert loaded.fused is False

    # Update fused status
    loaded.fused = True
    loaded.chip_info = "ESP32-C3 detected"
    loaded.save(metadata_path)

    # Load again and verify
    updated = KeyMetadata.from_file(metadata_path)
    assert updated.fused is True
    assert updated.chip_info == "ESP32-C3 detected"
