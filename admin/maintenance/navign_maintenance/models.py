"""Data models for key metadata and beacon registration."""

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class KeyMetadata:
    """
    Metadata for generated cryptographic keys.

    Stores key information, generation timestamp, and fusing status.
    """

    key_name: str
    private_key_file: str
    public_key_hex: str
    generated_at: str
    fused: bool = False
    chip_info: Optional[str] = None

    def to_json(self) -> str:
        """Serialize metadata to JSON string."""
        return json.dumps(
            {
                "key_name": self.key_name,
                "private_key_file": self.private_key_file,
                "public_key_hex": self.public_key_hex,
                "generated_at": self.generated_at,
                "fused": self.fused,
                "chip_info": self.chip_info,
            },
            indent=2,
        )

    @classmethod
    def from_json(cls, json_str: str) -> "KeyMetadata":
        """Deserialize metadata from JSON string."""
        data = json.loads(json_str)
        return cls(
            key_name=data["key_name"],
            private_key_file=data["private_key_file"],
            public_key_hex=data["public_key_hex"],
            generated_at=data["generated_at"],
            fused=data.get("fused", False),
            chip_info=data.get("chip_info"),
        )

    @classmethod
    def from_file(cls, path: Path) -> "KeyMetadata":
        """Load metadata from JSON file."""
        return cls.from_json(path.read_text())

    def save(self, path: Path) -> None:
        """Save metadata to JSON file."""
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(self.to_json())
