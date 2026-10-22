# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by art.whole_figure_lab.build
"""Preregistered attribute balance, fixed before any production sheets arrive."""
from dataclasses import dataclass
from typing import Final


@dataclass(frozen=True, slots=True)
class Person:
    """One anonymous review specimen and its private expected attributes."""
    sex: str
    age: str
    top: str
    hair: str
    activity: str

    @property
    def look(self) -> str:
        return f'{self.age}_{self.sex}_{self.top}_{self.hair}'


PEOPLE: Final = (
    Person('M', 'adult', 'hide_wrap', 'short', 'idle'),
    Person('F', 'adult', 'fur_cloak', 'long_tied', 'walk'),
    Person('M', 'adult', 'loincloth', 'bun', 'work'),
    Person('F', 'adult', 'hide_wrap', 'bun', 'carry'),
    Person('M', 'child', 'fur_cloak', 'short', 'sleep'),
    Person('F', 'child', 'loincloth', 'long_tied', 'walk'),
    Person('M', 'child', 'hide_wrap', 'long_tied', 'carry'),
    Person('F', 'child', 'fur_cloak', 'bun', 'work'),
    Person('F', 'adult', 'loincloth', 'short', 'sleep'),
    Person('M', 'adult', 'fur_cloak', 'long_tied', 'idle'),
    Person('F', 'child', 'hide_wrap', 'short', 'walk'),
    Person('M', 'child', 'loincloth', 'bun', 'carry'),
)
STATES: Final = ('idle', 'walk_a', 'work_a', 'carry', 'sleep')
FRAMES: Final = {
    'idle': (('idle', 0),) * 8,
    'walk': (('idle', 0), ('walk_a', -1)) * 4,
    'work': (('idle', 0), ('work_a', 0)) * 4,
    'carry': (('carry', 0), ('carry', -1)) * 4,
    'sleep': (('sleep', 0),) * 8,
}
