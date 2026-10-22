# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "playwright"]
# ///
# How to run: used by python3 -m art.whole_figure_lab.browser LAB capture
"""TRIAL3b real playback recorder with enough viewport for wrapped lab controls."""
import hashlib
import json
import os
import time
from pathlib import Path

from PIL import Image, ImageDraw
from playwright.sync_api import sync_playwright

from art.pipeline3.browser_lab import Specimen, loaded, serve


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
                                        'height': max(900, specimen.height + 360)})
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
