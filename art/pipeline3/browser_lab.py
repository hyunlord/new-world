# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "playwright"]
# ///
# How to run: python3 -m art.pipeline3.browser_lab GIF_DIRECTORY OUTPUT --capture
"""Publish anonymous GIFs and record actual Chromium playback, without seeking."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import threading
import time
from contextlib import contextmanager
from dataclasses import asdict, dataclass
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Final, Iterator

from PIL import Image, ImageDraw
from playwright.sync_api import Page, sync_playwright

SOURCE: Final = Path(__file__).parent


@dataclass(frozen=True, slots=True)
class Specimen:
    """Source identity with native pixel dimensions and anonymous public name."""

    id: str
    zoom: int
    file: str
    width: int
    height: int
    sha256: str
    source: str


def publish(gifs: Path, output: Path) -> list[Specimen]:
    """Validate all real inputs before publishing a viewer; never synthesize them."""
    specimens: list[Specimen] = []
    for zoom in (1, 2):
        for index in range(1, 13):
            filename = f'b_{index:02d}_{zoom}x.gif'
            source = gifs / filename
            with Image.open(source) as image:
                if image.format != 'GIF':
                    message = f'Expected GIF bytes: {source}'
                    raise ValueError(message)
                width, height = image.size
            specimens.append(Specimen(f'b_{index:02d}', zoom, filename, width, height,
                                      hashlib.sha256(source.read_bytes()).hexdigest(),
                                      str(source.resolve())))
    output.mkdir(parents=True, exist_ok=True)
    for specimen in specimens:
        destination = output / specimen.file
        if Path(specimen.source) != destination.resolve():
            shutil.copyfile(specimen.source, destination)
    for suffix in ('html', 'css', 'js'):
        shutil.copyfile(SOURCE / f'browser_lab.{suffix}', output / f'browser_lab.{suffix}')
    status_file = gifs.parent / 'DATASET_STATUS.txt'
    dataset_status = status_file.read_text().splitlines()[0] if status_file.exists() else 'UNVERIFIED_INPUTS'
    (output / 'manifest.json').write_text(json.dumps(
        {'dataset_status': dataset_status, 'specimens': [
            {key: value for key, value in asdict(item).items() if key != 'source'}
            for item in specimens]}, indent=2) + '\n')
    return specimens


@contextmanager
def serve(directory: Path) -> Iterator[str]:
    """Expose only the output directory on an ephemeral loopback port."""
    handler = partial(SimpleHTTPRequestHandler, directory=str(directory.resolve()))
    with ThreadingHTTPServer(('127.0.0.1', 0), handler) as server:
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            yield f'http://127.0.0.1:{server.server_port}/browser_lab.html'
        finally:
            server.shutdown()
            thread.join()


def loaded(page: Page, url: str) -> None:
    """Require decoded source images before measuring live rendering."""
    page.goto(url, wait_until='networkidle')
    page.wait_for_function('window.labReady === true')
    page.wait_for_function('''Array.from(document.images).every(
        image => image.complete && image.naturalWidth > 0)''')


def capture(output: Path, specimens: list[Specimen], seconds: float) -> None:
    """Capture native browser frames on a target 125 ms cadence with actual times."""
    evidence = output / 'playback'
    evidence.mkdir(exist_ok=True)
    records = []
    with serve(output) as url, sync_playwright() as playwright:
        browser = playwright.chromium.launch(executable_path=os.environ.get('CHROMIUM_EXECUTABLE'))
        try:
            page = browser.new_page(viewport={'width': 1280, 'height': 900}, device_scale_factor=1)
            errors: list[str] = []
            page.on('pageerror', lambda error: errors.append(str(error)))
            for specimen in specimens:
                loaded(page, f'{url}?id={specimen.id}&zoom={specimen.zoom}')
                page.set_viewport_size({'width': max(375, specimen.width + 32),
                                        'height': max(480, specimen.height + 240)})
                destination = evidence / f'{specimen.id}_{specimen.zoom}x'
                destination.mkdir(exist_ok=True)
                image_box = page.locator('img').bounding_box()
                if image_box is None:
                    message = 'Displayed GIF has no browser image bounds'
                    raise RuntimeError(message)
                start = time.perf_counter()
                samples = []
                for index in range(round(seconds * 8)):
                    delay = start + index / 8 - time.perf_counter()
                    if delay > 0:
                        page.wait_for_timeout(delay * 1000)
                    before = time.perf_counter() - start
                    png = page.screenshot(path=str(destination / f'{index:03d}.png'))
                    after = time.perf_counter() - start
                    samples.append({'index': index, 'before_s': before, 'after_s': after,
                                    'sha256': hashlib.sha256(png).hexdigest()})
                # These crops come from actual browser screenshots, never decoded GIF frames.
                crop = (round(image_box['x']), round(image_box['y']),
                        round(image_box['x'] + image_box['width']),
                        round(image_box['y'] + image_box['height']))
                for first in range(0, len(samples), 8):
                    contact = Image.new('RGB', (specimen.width * 4, (specimen.height + 20) * 2), '#e0bd8e')
                    draw = ImageDraw.Draw(contact)
                    for slot, sample in enumerate(samples[first:first + 8]):
                        x, y = slot % 4 * specimen.width, slot // 4 * (specimen.height + 20)
                        with Image.open(destination / f"{sample['index']:03d}.png") as screenshot:
                            contact.paste(screenshot.crop(crop), (x, y))
                        draw.text((x + 2, y + specimen.height), f"{sample['before_s']:.3f}s", fill='#18110a')
                    contact.save(destination / f'watched_contact_{first // 8 + 1:02d}.png')
                unique = len({sample['sha256'] for sample in samples})
                records.append({'id': specimen.id, 'zoom': specimen.zoom,
                                'source_sha256': specimen.sha256, 'image_box': image_box, 'samples': samples,
                                'unique_screenshots': unique,
                                'motion': 'VISIBLE_CHANGE' if unique > 1 else 'STATIC_OR_UNOBSERVED'})
            overview = output / 'overview'
            overview.mkdir(exist_ok=True)
            for width in (375, 768, 1280):
                page.set_viewport_size({'width': width, 'height': 900})
                loaded(page, url)
                page.screenshot(path=str(overview / f'{width}.png'), full_page=True)
            page.select_option('#ground', 'forest')
            page.uncheck('#grid')
            page.select_option('#zoom', '2')
            page.wait_for_function('Array.from(document.images).every(i => i.complete && i.naturalWidth > 0)')
            page.screenshot(path=str(overview / 'forest_2x_outlines_off.png'), full_page=True)
            (output / 'playback.json').write_text(json.dumps({
                'browser': browser.version, 'device_scale_factor': 1,
                'target_hz': 8, 'requested_seconds_per_gif': seconds,
                'method': 'native GIF playback; no seek, canvas or decoded-frame substitution',
                'page_errors': errors, 'specimens': records,
                'review_scores': 'NOT_SCORED',
            }, indent=2) + '\n')
        finally:
            browser.close()


def crowd(output: Path) -> None:
    """Measure 200 native GIF elements for 60s; explicitly not a Godot FPS result."""
    with serve(output) as url, sync_playwright() as playwright:
        browser = playwright.chromium.launch(executable_path=os.environ.get('CHROMIUM_EXECUTABLE'))
        try:
            page = browser.new_page(viewport={'width': 1600, 'height': 1000})
            loaded(page, url)
            result = page.evaluate('''async () => {
                const root = document.querySelector('#specimens');
                const source = root.querySelector('img');
                const width = source.naturalWidth, height = source.naturalHeight;
                root.replaceChildren();
                for (let index = 0; index < 200; index++) {
                    const img = source.cloneNode(); root.append(img);
                }
                const deltas = []; let previous, start;
                await new Promise(resolve => {
                    function tick(now) {
                        if (start === undefined) start = now;
                        if (previous !== undefined) deltas.push(now - previous);
                        previous = now;
                        if (now - start >= 60000) resolve();
                        else requestAnimationFrame(tick);
                    }
                    requestAnimationFrame(tick);
                });
                return {deltas_ms: deltas, elapsed_ms: previous - start,
                    count: root.querySelectorAll('img').length, width, height};
            }''')
            (output / 'crowd_browser.json').write_text(json.dumps({
                'surface': 'Chromium DOM GIF scheduling, NOT Godot P1',
                'limitations': 'Images can exceed viewport; rAF does not prove 200 visible sprites.',
                'measurement': result,
            }, indent=2) + '\n')
        finally:
            browser.close()


def main() -> None:
    """Keep CLI stdlib-only, consistent with existing pipeline tools."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('gifs', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--capture', action='store_true')
    parser.add_argument('--seconds', type=float, default=3.0)
    parser.add_argument('--crowd', action='store_true')
    args = parser.parse_args()
    if args.seconds < 1:
        parser.error('--seconds must be at least 1')
    specimens = publish(args.gifs, args.output)
    if args.capture:
        capture(args.output, specimens, args.seconds)
    if args.crowd:
        crowd(args.output)
    print(args.output / 'browser_lab.html')


if __name__ == '__main__':
    main()
