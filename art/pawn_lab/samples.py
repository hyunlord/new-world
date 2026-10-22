# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by art.pawn_lab.build
"""Preregistered twelve-person attribute balance and activity sequences."""
from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class Person:
    sex: str
    age: str
    skin_tone: int
    hair_style: str
    hair_color: int
    top: str
    top_palette: int
    item: str
    activity: str


PEOPLE = (
    Person('M', 'adult', 0, 'short', 0, 'hide_wrap', 0, 'none', 'idle'),
    Person('F', 'adult', 1, 'long_tied', 1, 'woven_tunic', 1, 'basket', 'walk'),
    Person('M', 'adult', 2, 'none', 2, 'fur_cloak', 0, 'stone_axe', 'work'),
    Person('F', 'adult', 3, 'bun', 0, 'hide_wrap', 1, 'log_bundle', 'carry'),
    Person('M', 'child', 1, 'short', 1, 'woven_tunic', 0, 'none', 'sleep'),
    Person('F', 'child', 2, 'braid', 2, 'fur_cloak', 1, 'basket', 'walk'),
    Person('M', 'elder', 3, 'grey_thin', 3, 'hide_wrap', 0, 'log_bundle', 'carry'),
    Person('F', 'elder', 0, 'grey_thin', 3, 'woven_tunic', 1, 'stone_axe', 'work'),
    Person('F', 'adult', 2, 'long_tied', 2, 'none', 0, 'none', 'sleep'),
    Person('M', 'adult', 1, 'bun', 0, 'fur_cloak', 1, 'basket', 'idle'),
    Person('F', 'child', 0, 'braid', 1, 'hide_wrap', 1, 'none', 'walk'),
    Person('M', 'elder', 2, 'grey_thin', 3, 'woven_tunic', 0, 'log_bundle', 'carry'),
)
FRAMES = {'idle': ('idle',) * 8, 'walk': ('walk_a', 'walk_b') * 4,
          'work': ('work_a', 'work_b') * 4, 'carry': ('carry',) * 8,
          'sleep': ('sleep',) * 8}
