# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow"]
# ///
# How to run: imported by python3 -m art.flora.boards
"""Lossless board persistence with all compression diagnostics in ART-TREES data."""
import shutil
import subprocess
import tempfile
from pathlib import Path

from PIL import Image


def save_lossless(image: Image.Image, path: Path, *, optimize_oversize: bool = False) -> None:
    """Enforce the file cap, verifying decoded pixels after optional PNG refiltering."""
    image.save(path, optimize=True, compress_level=9)
    if optimize_oversize and path.stat().st_size > 1_500_000:
        data = Path("/Users/rexxa/work/arttrees-data/boards")
        data.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="png-lossless-", dir=data) as directory:
            compressed = Path(directory) / "compressed.png"
            _ = subprocess.run(["xcrun", "pngcrush", "-q", "-brute",
                                str(path), str(compressed)], check=True, capture_output=True, timeout=180)
            with Image.open(compressed) as decoded:
                if decoded.convert("RGBA").tobytes() != image.convert("RGBA").tobytes():
                    message = f"Lossless compression changed pixels: {path}"
                    raise ValueError(message)
            if compressed.stat().st_size < path.stat().st_size:
                _ = shutil.copyfile(compressed, path)
    if path.stat().st_size > 1_500_000:
        message = f"Board exceeds 1.5MB after lossless compression: {path}: {path.stat().st_size}"
        raise ValueError(message)
