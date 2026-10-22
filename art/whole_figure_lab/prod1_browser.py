# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "playwright"]
# ///
# How to run: python3 -m art.whole_figure_lab.browser LAB [capture|crowd|publish]
"""Reuse native browser playback capture and measure a fully visible sprite crowd."""
from __future__ import annotations

import json
import os
import shutil
import sys
from dataclasses import asdict
from pathlib import Path

from PIL import Image
from playwright.sync_api import sync_playwright

from art.pipeline3.browser_lab import Specimen, serve

from .build import sha
from .capture import capture


def publish(lab: Path) -> list[Specimen]:
    """Publish no answer key or source attribute names into the blind viewer."""
    output = lab / 'browser'
    output.mkdir(exist_ok=True)
    specimens = []
    for zoom in (1, 2):
        for index in range(1, 13):
            filename = f'p1_{index:02d}_{zoom}x.gif'
            source = lab / 'gifs' / filename
            with Image.open(source) as image:
                width, height = image.size
            specimens.append(Specimen(f'p1_{index:02d}', zoom, filename, width, height, sha(source), str(source)))
            shutil.copyfile(source, output / filename)
    for suffix in ('html', 'css', 'js'):
        shutil.copyfile(Path(__file__).with_name(f'browser_lab.{suffix}'), output / f'browser_lab.{suffix}')
    (output / 'manifest.json').write_text(json.dumps({
        'dataset_status': (lab / 'DATASET_STATUS.txt').read_text().splitlines()[0],
        'specimens': [{key: value for key, value in asdict(item).items() if key != 'source'} for item in specimens],
    }, indent=2) + '\n')
    return specimens


def crowd(lab: Path) -> None:
    """Draw all 200 at 1× for 60s; screenshots prove on-canvas coverage."""
    output = lab / 'crowd'
    output.mkdir(exist_ok=True)
    shutil.copytree(lab / 'textures', output / 'textures', dirs_exist_ok=True)
    for filename in ('crowd_manifest.json', 'meadow.png'):
        shutil.copyfile(lab / filename, output / filename)
    shutil.copyfile(Path(__file__).with_name('prod1_crowd.js'), output / 'crowd.js')
    (output / 'browser_lab.html').write_text('<!doctype html><html lang="en"><meta charset="utf-8"><title>Whole-figure crowd</title><body><script src="crowd.js"></script></body></html>')
    with serve(output) as url, sync_playwright() as playwright:
        browser = playwright.chromium.launch(executable_path=os.environ.get('CHROMIUM_EXECUTABLE'))
        try:
            page = browser.new_page(viewport={'width': 2240, 'height': 1120}, device_scale_factor=1)
            errors: list[str] = []
            page.on('pageerror', lambda error: errors.append(str(error)))
            page.goto(url)
            page.wait_for_function('window.crowdReady || window.crowdError')
            page.screenshot(path=str(output / 'start.png'))
            page.wait_for_function('window.crowdResult || window.crowdError', timeout=75000)
            page.screenshot(path=str(output / 'end.png'))
            result = page.evaluate('({measurement:window.crowdResult,error:window.crowdError || null})')
            (output / 'runtime.json').write_text(json.dumps({'browser': browser.version,
                'device_scale_factor': 1, 'page_errors': errors, **result,
                'source_hashes': {str(path.relative_to(output)): sha(path) for path in sorted((output / 'textures').glob('*.png'))},
                'limitations': 'Isolated Chromium Canvas2D renderer, not Godot, not simulation. rAF completion measures submitted canvas frames, not display scanout.'}, indent=2) + '\n')
            if errors or result['error'] or not result['measurement']:
                message = f'Crowd rendering failed: {errors}, {result["error"]}'
                raise RuntimeError(message)
        finally:
            browser.close()


if __name__ == '__main__':
    lab = Path(sys.argv[1])
    mode = sys.argv[2] if len(sys.argv) > 2 else 'publish'
    specimens = publish(lab)
    if mode == 'capture':
        capture(lab / 'browser', specimens, 3.0)
    if mode == 'crowd':
        crowd(lab)
