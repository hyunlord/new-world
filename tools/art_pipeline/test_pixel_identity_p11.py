"""Exercise the P5 exact-control rule through real RGBA image comparisons."""
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from PIL import Image

from pixel_identity_p11 import Capture, Difference, compare


class PixelIdentityP11Tests(unittest.TestCase):
    def test_exact_p5_control_with_real_rgba_images(self) -> None:
        cases = [
            ('P5', True, False, False),
            ('P5', True, True, False),
            ('P5', False, True, False),
            ('P5', False, False, True),
            ('P1', True, False, True),
            ('P1', True, True, True),
            ('P1', False, True, False),
        ]
        for framing, repeat_changed, candidate_changed, expected in cases:
            with self.subTest(
                framing=framing,
                repeat_changed=repeat_changed,
                candidate_changed=candidate_changed,
            ), TemporaryDirectory() as temporary:
                tmp_path = Path(temporary)
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
                self.assertEqual(result.control, Difference(int(repeat_changed), int(repeat_changed)))
                self.assertEqual(result.candidate, Difference(int(candidate_changed), int(candidate_changed)))
                self.assertIs(result.passed, expected)
                self.assertIs(result.bit_reproducible, not repeat_changed)
                if framing == 'P5' and repeat_changed:
                    self.assertIn('P5 requires exact', result.cause)
                with Image.open(tmp_path / f'{framing}-control-diff.png') as image:
                    self.assertEqual(image.getpixel((0, 0)), int(repeat_changed))


if __name__ == "__main__":
    unittest.main()
