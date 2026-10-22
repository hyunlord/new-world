# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "playwright"]
# ///
# How to run: python3 -m art.pawn_lab.browser LAB_OUTPUT [SECONDS]
"""Publish pawn GIFs through the existing native Chromium playback recorder."""
import hashlib
import json
import shutil
import sys
from dataclasses import asdict
from pathlib import Path
from PIL import Image
from art.pipeline3.browser_lab import Specimen, capture


def publish(lab: Path) -> list[Specimen]:
    """Public page contains only anonymous numbers, scales and source pixels."""
    output = lab / 'browser'
    output.mkdir(exist_ok=True)
    specimens = []
    for zoom in (1, 2):
        for index in range(1, 13):
            filename = f'p_{index:02d}_{zoom}x.gif'
            source = lab / 'gifs' / filename
            with Image.open(source) as image:
                width, height = image.size
            item = Specimen(f'p_{index:02d}', zoom, filename, width, height,
                            hashlib.sha256(source.read_bytes()).hexdigest(), str(source))
            specimens.append(item)
            shutil.copyfile(source, output / filename)
    for suffix in ('html', 'css', 'js'):
        shutil.copyfile(Path(__file__).with_name(f'browser_lab.{suffix}'), output / f'browser_lab.{suffix}')
    (output / 'manifest.json').write_text(json.dumps({
        'dataset_status': (lab / 'DATASET_STATUS.txt').read_text().splitlines()[0],
        'specimens': [{k: v for k, v in asdict(item).items() if k != 'source'} for item in specimens],
    }, indent=2) + '\n')
    return specimens


if __name__ == '__main__':
    root = Path(sys.argv[1])
    capture(root / 'browser', publish(root), float(sys.argv[2]) if len(sys.argv) > 2 else 3.0)
