# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by python -m art.pipeline5.sheet_slice
"""Publish a validated complete sheet by one sibling-directory rename."""
import shutil
import tempfile
from pathlib import Path

from art.pipeline5.attempt_integrity import validate_bundle
from art.pipeline5.whole_checks import FRAMES


def publish(diagnostic: Path, target: Path) -> None:
    """A failed copy or validation never exposes a partial public sheet."""
    if target.exists():
        message = f'delivery exists; preserve it: {target}'
        raise ValueError(message)
    target.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix='.'+target.name+'-staging-', dir=target.parent))
    try:
        for frame in FRAMES:
            for suffix in ('.png', '.json'):
                _ = shutil.copyfile(diagnostic/(frame+suffix), stage/(frame+suffix))
        validate_bundle(stage, diagnostic/'sheet_checks.json')
        stage.rename(target)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
