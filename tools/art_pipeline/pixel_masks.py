# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: import make_mask from pixel_masks
"""Deterministic authored ASCII silhouettes for small production fallbacks."""
from __future__ import annotations
from PIL import Image
from palette_tools import Color, rgb_to_lab
import numpy as np

MASKS: dict[str, tuple[str, ...]] = {
 'grass': ('...a...', '.a.a.a.', '..aaa..', '...a...'),
 'flower': ('..aaa..', '.ababa.', '..aaa..', '...c...', '..cc...'),
 'mushroom': ('..aaa..', '.abbba.', 'aaaaaaa', '..cc...'),
 'pebble': ('..aaa..', '.abbba.', 'aaaaaaa'),
 'leaf': ('....a..', '..aaba.', '.abba..', '..a....'),
 'lily': ('..aaa..', '.abbba.', 'aa.bbaa', '.aaaaa.'),
 'smoke': ('...aaa.', '.aabbba', 'abbbbba', '.abbaa.', '..aaa..'),
 'spark': ('...a...', '.aaba..', '...a...', '..a....'),
 'dust': ('.......', '..a.a..', '.abbba.', 'aa.a.aa'),
 'splash': ('a..a..a', '.a.a.a.', '..aaa..', '.abbba.'),
 'sparkle': ('...a...', '...b...', '.ababa.', '...b...', '...a...'),
 'sleep': ('aaaa...', '..a....', '.a.....', 'aaaa...', '....aaa', '.....a.', '....aaa'),
 'bird': ('aa...aa', '.aa.aa.', '..aba..', '...a...'),
 'butterfly': ('aa...aa', 'aba.aba', '.aabaa.', '...a...'),
 'fish': ('..aaa..', 'aabbbaa', '..aaa..'),
 'dragonfly': ('aa...aa', '..aaa..', 'aa.a.aa', '...a...'),
 'people': ('..aa...', '..bb...', '.aaaa..', 'aaabaa.', '..aa...', '.a..a..'),
 'hearth': ('...a...', '..aba..', '.ababa.', '..aba..', 'ccccccc'),
 'shelter': ('...a...', '..aba..', '.abbba.', 'abbbbba', 'aa..aaa'),
 'hammer': ('..aaaa.', '..abba.', '...a...', '..a....', '.a.....'),
 'basket': ('.aaaaa.', 'a.....a', 'abbbbba', '.ababa.', '..aaa..'),
 'berries': ('..c.c..', '.aaa...', '.aba.aa', '..a.aba', '.....a.'),
 'sprout': ('aa...aa', 'aba.aba', '..aaa..', '...a...', '..aaa..'),
 'cairn': ('...a...', '..aba..', '.aaaaa.', 'aabbbaa'),
 'sun': ('a..a..a', '..aaa..', '.ababa.', 'aaaaaaa', '.ababa.', '..aaa..', 'a..a..a'),
 'moon': ('..aaa..', '.abba..', 'abba...', 'abba...', '.abba..', '..aaaa.'),
 'water': ('...a...', '..aba..', '.abbba.', 'abbbbba', '.aaaaa.'),
 'pause': ('.aa.aa.', '.aa.aa.', '.aa.aa.', '.aa.aa.', '.aa.aa.'),
 'fast': ('a..a...', 'aa.aa..', 'aaaaaaa', 'aa.aa..', 'a..a...'),
 'scroll': ('.aaaaaa', 'aabbbba', '.abbbba', '.abbbba', '.aaaaaa'),
}

def make_mask(name: str, palette: list[Color], size: tuple[int, int] = (16, 16), variant: int = 0) -> Image.Image:
    """Render a named ASCII mask centred in an exact-size binary-alpha canvas."""
    pattern = MASKS[name]
    labs = rgb_to_lab(np.asarray(palette, dtype=np.uint8))
    order = np.argsort(labs[:, 0])
    colors = {'a': palette[int(order[0])], 'b': palette[int(order[len(order) * 2 // 3])],
              'c': palette[int(order[len(order) // 3])]}
    image = Image.new('RGBA', size)
    ox, oy = (size[0] - len(pattern[0])) // 2, (size[1] - len(pattern)) // 2
    for y, row in enumerate(pattern):
        for x, key in enumerate(row):
            if key in colors:
                px = ox + (len(row) - 1 - x if variant % 2 else x)
                py = oy + y
                if 0 <= px < size[0] and 0 <= py < size[1]:
                    image.putpixel((px, py), (*colors[key], 255))
    return image
