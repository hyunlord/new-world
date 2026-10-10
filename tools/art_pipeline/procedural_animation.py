# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: import animate from procedural_animation
"""Exact-pixel pose fallbacks; no resizing, interpolation or invented generation."""
from __future__ import annotations
import numpy as np
from PIL import Image


def _shift_region(image: Image.Image, box: tuple[int, int, int, int], dx: int, dy: int) -> Image.Image:
    result = image.copy()
    region = image.crop(box)
    result.paste((0, 0, 0, 0), box)
    result.alpha_composite(region, (box[0] + dx, box[1] + dy))
    return result


def animate(idle: Image.Image, activity: str, frame: int, view: str = 'S',
            basket: Image.Image | None = None, bedroll: Image.Image | None = None) -> Image.Image:
    """Pose an unoutlined accepted idle; caller records rule and adds outline."""
    image = idle.convert('RGBA')
    w, h = image.size
    match activity:
        case 'walk' | 'carry' | 'run' | 'hop':
            index = frame % 4
            if index in (1, 3):
                result = _shift_region(image, (0, 0, w, h), 0, -1)
            else:
                x0, x1 = (0, w // 2) if index == 0 else (w // 2, w)
                result = _shift_region(image, (x0, int(h * .7), x1, h), 0, -1)
            if activity == 'carry' and view != 'N' and basket is not None:
                result.alpha_composite(basket.convert('RGBA'), ((w - basket.width) // 2, int(h * .55) - basket.height // 2))
            return result
        case 'gather' | 'build' | 'crouch':
            result = _shift_region(image, (0, 0, w, int(h * .6)), 1 if frame % 2 == 0 else 0, 0)
            if activity == 'build':
                pixels = np.asarray(image)
                opaque = pixels[pixels[..., 3] == 255]
                color = tuple(map(int, opaque[np.argmin(opaque[:, :3].sum(1))])) if len(opaque) else (40, 30, 20, 255)
                glyph = Image.new('RGBA', (3, 3), color)
                result.alpha_composite(glyph, (w // 2, int(h * .55) + (-2 if frame % 2 == 0 else 2)))
            return result
        case 'talk' | 'idle' | 'crossed':
            return _shift_region(image, (0, 0, w, int(h * .35)), 0, -1 if frame % 2 else 0)
        case 'eat' | 'drink' | 'sit' | 'hug':
            result = _shift_region(image, (0, 0, w, int(h * .6)), 0, 2)
            split = int(h * .8)
            result = Image.fromarray(np.concatenate((np.asarray(result)[:split], np.asarray(result)[split + 2:]), axis=0))
            if activity in ('sit', 'hug'):
                rgba = np.array(result)
                start = int(result.height * .7)
                leg = rgba[start:]
                colors = leg[leg[..., 3] == 255]
                rgba[start:] = 0
                if len(colors):
                    rgba[-2:, 1:-1] = colors[len(colors) // 2]
                result = Image.fromarray(rgba)
            return result
        case 'asleep':
            body = image.transpose(Image.Transpose.ROTATE_270)
            if bedroll is None:
                return body
            result = Image.new('RGBA', (max(body.width, bedroll.width), max(body.height, bedroll.height)))
            result.alpha_composite(bedroll.convert('RGBA'), ((result.width - bedroll.width) // 2, result.height - bedroll.height))
            result.alpha_composite(body, ((result.width - body.width) // 2, result.height - body.height))
            return result
        case _:
            raise ValueError(f'Unsupported fallback activity: {activity}')
