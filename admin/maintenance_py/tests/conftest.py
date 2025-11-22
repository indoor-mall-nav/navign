"""Test configuration and fixtures."""

import pytest
from pathlib import Path


@pytest.fixture
def temp_keys_dir(tmp_path):
    """Create a temporary directory for key files."""
    keys_dir = tmp_path / "keys"
    keys_dir.mkdir()
    return keys_dir


@pytest.fixture
def sample_private_key():
    """Generate a sample 32-byte private key."""
    import secrets

    return secrets.token_bytes(32)


@pytest.fixture
def sample_public_key_hex():
    """Sample uncompressed P-256 public key hex (130 chars)."""
    # This is a valid public key format (0x04 + 64 hex bytes)
    return "04" + "a1" * 32 + "b2" * 32
