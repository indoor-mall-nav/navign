"""
Cryptographic key generation and management for ESP32-C3 beacons.

Provides P-256 ECDSA key pair generation, PEM conversion, and hex encoding
compatible with the Navign beacon system.
"""

import secrets
from datetime import datetime, timezone
from pathlib import Path
from typing import Tuple

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ec


def generate_p256_key_pair() -> Tuple[bytes, bytes, str]:
    """
    Generate a P-256 ECDSA private/public key pair.

    Returns:
        Tuple of (private_key_bytes, public_key_bytes, public_key_hex)
        - private_key_bytes: 32-byte private key (raw)
        - public_key_bytes: 65-byte uncompressed public key (SEC1)
        - public_key_hex: Hex-encoded public key (130 characters)
    """
    # Generate private key using P-256 (secp256r1) curve
    private_key = ec.generate_private_key(ec.SECP256R1())

    # Extract raw private key bytes (32 bytes)
    private_key_bytes = private_key.private_numbers().private_value.to_bytes(32, "big")

    # Get public key
    public_key = private_key.public_key()

    # Export uncompressed public key (0x04 + x + y = 65 bytes)
    public_key_bytes = public_key.public_bytes(
        encoding=serialization.Encoding.X962,
        format=serialization.PublicFormat.UncompressedPoint,
    )

    # Convert to hex string
    public_key_hex = public_key_bytes.hex()

    return private_key_bytes, public_key_bytes, public_key_hex


def public_key_to_pem(public_key_bytes: bytes) -> str:
    """
    Convert public key bytes to PEM format for gRPC transmission.

    Args:
        public_key_bytes: 65-byte uncompressed SEC1 public key

    Returns:
        PEM-encoded public key string
    """
    # Load public key from bytes
    public_key = ec.EllipticCurvePublicKey.from_encoded_point(
        ec.SECP256R1(), public_key_bytes
    )

    # Export as PEM
    pem = public_key.public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo,
    )

    return pem.decode("utf-8")


def generate_device_id() -> str:
    """
    Generate a random 24-character hexadecimal device ID (12 bytes).

    Returns:
        24-character hex string (e.g., "a1b2c3d4e5f6g7h8i9j0k1l2")
    """
    random_bytes = secrets.token_bytes(12)
    return random_bytes.hex()


def save_private_key(key_bytes: bytes, output_path: Path) -> None:
    """
    Save private key bytes to a binary file.

    Args:
        key_bytes: 32-byte private key
        output_path: Path to save the file

    Raises:
        IOError: If file cannot be written
    """
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_bytes(key_bytes)


def get_timestamp_rfc3339() -> str:
    """
    Get current timestamp in RFC 3339 / ISO 8601 format.

    Returns:
        Timestamp string (e.g., "2025-01-15T10:30:00Z")
    """
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
