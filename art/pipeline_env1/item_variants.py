# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow"]
# ///
# How to run: PYTHONPATH=. uv run art/pipeline_env1/item_variants.py
from __future__ import annotations

import json
from hashlib import sha256
from pathlib import Path

from PIL import Image

from art.pipeline_env1.items_records import JsonRecord

ROOT = Path(__file__).resolve().parents[2]
FACTORS = (0.55, 0.70, 0.45)


def spoiled_meat(image: Image.Image) -> Image.Image:
    red, green, blue, alpha = image.convert("RGBA").split()
    channels = tuple(Image.frombytes("L", channel.size, channel.tobytes().translate(
                         bytes(round(value * factor) for value in range(256))))
                     for channel, factor in zip((red, green, blue), FACTORS, strict=True))
    return Image.merge("RGBA", (*channels, alpha))


def main() -> None:
    source = ROOT / "art/style05/env1/items/meat.png"
    target = source.with_name("meat_spoiled.png")
    original = Image.open(source).convert("RGBA")
    result = spoiled_meat(original)
    if result.getchannel("A").tobytes() != original.getchannel("A").tobytes():
        raise ValueError("Global tone must preserve every alpha byte")
    result.save(target, optimize=True, compress_level=9)
    evidence = {
        "source": str(source.relative_to(ROOT)),
        "source_sha256": sha256(source.read_bytes()).hexdigest(),
        "path": str(target.relative_to(ROOT)),
        "sha256": sha256(target.read_bytes()).hexdigest(),
        "srgb_multiplier": list(FACTORS), "rounding": "nearest integer, Python round",
        "alpha": "byte-identical", "canvas": list(result.size),
        "method": "code-only whole-sprite RGB lookup table; no palette quantization",
        "status": "DERIVED_CANDIDATE_NOT_IN_BLIND_ITEM_PACKET",
        "generation_calls": 0,
        "note": "Declared art state; native simulation removes expired meat.",
    }
    _ = target.with_suffix(".json").write_text(json.dumps(evidence, indent=2) + "\n")
    manifest_path = ROOT / "docs/v8/artenv1/items_manifest.json"
    manifest = JsonRecord.read(manifest_path)
    manifest.fields["spoiled_meat"] = evidence
    _ = manifest_path.write_text(json.dumps(manifest.fields, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps(evidence, indent=2))


if __name__ == "__main__":
    main()
