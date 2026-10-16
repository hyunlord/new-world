from pathlib import Path

import pytest
from PIL import Image

from pixel_identity_p11 import Capture, Difference, compare


@pytest.mark.parametrize(
    ('framing', 'repeat_changed', 'candidate_changed', 'expected'),
    [
        ('P5', True, False, False),
        ('P5', True, True, False),
        ('P5', False, True, False),
        ('P5', False, False, True),
        ('P1', True, False, True),
        ('P1', True, True, True),
        ('P1', False, True, False),
    ],
)
def test_exact_p5_control_with_real_rgba_images(
    tmp_path: Path,
    framing: str,
    repeat_changed: bool,
    candidate_changed: bool,
    expected: bool,
) -> None:
    (tmp_path / 'previews').mkdir()
    captures: list[Capture] = []
    for label, changed in (('A', False), ('Aprime', repeat_changed), ('B', candidate_changed)):
        folder = tmp_path / label
        folder.mkdir()
        path = folder / f'{framing}.png'
        with Image.new('RGBA', (2, 2), (10, 20, 30, 255)) as image:
            if changed:
                image.putpixel((0, 0), (10, 20, 30, 254))
            image.save(path)
        captures.append(Capture(path, 130, 10, 1440, 15100, 60))
    result = compare((captures[0], captures[1], captures[2]), tmp_path)
    assert result.control == Difference(int(repeat_changed), int(repeat_changed))
    assert result.candidate == Difference(int(candidate_changed), int(candidate_changed))
    assert result.passed is expected
    assert result.bit_reproducible is (not repeat_changed)
    if framing == 'P5' and repeat_changed:
        assert 'P5 requires exact' in result.cause
    with Image.open(tmp_path / f'{framing}-control-diff.png') as image:
        assert image.getpixel((0, 0)) == int(repeat_changed)
