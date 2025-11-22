"""Tests for cryptographic key generation and management."""

from navign_maintenance import crypto


def test_generate_p256_key_pair():
    """Test P-256 key pair generation."""
    private_key_bytes, public_key_bytes, public_key_hex = (
        crypto.generate_p256_key_pair()
    )

    # Private key should be 32 bytes
    assert len(private_key_bytes) == 32

    # Public key should be 65 bytes (uncompressed: 0x04 + x + y)
    assert len(public_key_bytes) == 65
    assert public_key_bytes[0] == 0x04

    # Public key hex should be 130 characters
    assert len(public_key_hex) == 130
    assert public_key_hex.startswith("04")
    assert all(c in "0123456789abcdef" for c in public_key_hex)


def test_public_key_to_pem():
    """Test public key PEM conversion."""
    # Generate a key pair
    _, public_key_bytes, _ = crypto.generate_p256_key_pair()

    # Convert to PEM
    pem = crypto.public_key_to_pem(public_key_bytes)

    # Verify PEM format
    assert "-----BEGIN PUBLIC KEY-----" in pem
    assert "-----END PUBLIC KEY-----" in pem
    assert isinstance(pem, str)


def test_generate_device_id():
    """Test device ID generation."""
    device_id = crypto.generate_device_id()

    # Should be 24 characters (12 bytes in hex)
    assert len(device_id) == 24
    assert all(c in "0123456789abcdef" for c in device_id)

    # Multiple calls should generate different IDs
    device_id2 = crypto.generate_device_id()
    assert device_id != device_id2


def test_save_private_key(temp_keys_dir):
    """Test private key file saving."""
    key_bytes = b"a" * 32
    output_path = temp_keys_dir / "test_key.bin"

    crypto.save_private_key(key_bytes, output_path)

    # Verify file exists and contains correct data
    assert output_path.exists()
    assert output_path.read_bytes() == key_bytes


def test_get_timestamp_rfc3339():
    """Test RFC 3339 timestamp generation."""
    timestamp = crypto.get_timestamp_rfc3339()

    # Verify format
    assert "T" in timestamp
    assert timestamp.endswith("Z") or "+" in timestamp or "-" in timestamp

    # Verify can be parsed
    from datetime import datetime

    parsed = datetime.fromisoformat(timestamp.replace("Z", "+00:00"))
    assert parsed.timestamp() > 0


def test_public_key_round_trip():
    """Test that public key can be converted to PEM and back."""
    from cryptography.hazmat.primitives import serialization
    from cryptography.hazmat.primitives.asymmetric import ec

    # Generate key pair
    _, public_key_bytes, public_key_hex = crypto.generate_p256_key_pair()

    # Convert to PEM
    pem = crypto.public_key_to_pem(public_key_bytes)

    # Parse PEM back
    from cryptography.hazmat.backends import default_backend

    loaded_key = serialization.load_pem_public_key(
        pem.encode("utf-8"), backend=default_backend()
    )

    # Verify it's still a P-256 key
    assert isinstance(loaded_key.curve, ec.SECP256R1)

    # Convert back to bytes and compare
    loaded_bytes = loaded_key.public_bytes(
        encoding=serialization.Encoding.X962,
        format=serialization.PublicFormat.UncompressedPoint,
    )

    assert loaded_bytes == public_key_bytes
