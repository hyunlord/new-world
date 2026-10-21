"""Synthetic-only registered sheet tests; no fixture is an accepted art asset."""
import json
import tempfile
import unittest
from pathlib import Path

from PIL import Image, ImageDraw

from .frames import (
    SheetTransform,
    apply_transform,
    build_transform,
    extract_variants,
    normalize_cell,
    save_frames,
)
from .models import PipelineError, decode_json, json_array, json_object, json_string
from .pixelize import pixelize


def fixture(size: tuple[int, int] = (256, 256)) -> Image.Image:
    image = Image.new('RGBA', size, '#FF00FF')
    draw = ImageDraw.Draw(image)
    for row in range(2):
        for col in range(4):
            left, top = col * size[0] // 4, row * size[1] // 4
            draw.rectangle((left + 24, top + 10, left + 39, top + 49), fill='#9A9A9A')
            draw.rectangle((left + 24, top + 30, left + 39, top + 36), fill='#6E6E6E')
    top = 2 * size[1] // 4
    draw.rectangle((10, top + 24, 49, top + 39), fill='#9A9A9A')
    return image


class FrameTests(unittest.TestCase):
    def test_normalization_matches_general_majority(self) -> None:
        for size in ((8, 9), (9, 8), (9, 9)):
            image = Image.new('RGBA', size)
            image.putdata([((i % 5) * 40, (i % 3) * 70, 20, 255)
                           for i in range(size[0] * size[1])])
            self.assertEqual(normalize_cell(image, (8, 8)).tobytes(),
                             pixelize(image, (8, 8)).tobytes())

    def test_roundtrip_and_common_registration(self) -> None:
        image = fixture((257, 259))
        transform = build_transform(image)
        self.assertEqual(transform, SheetTransform.from_json(transform.to_json()))
        frames = apply_transform(image, transform)
        self.assertEqual(len(frames), 9)
        self.assertTrue(all(frame.image.size == (32, 32) for frame in frames[:8]))
        self.assertEqual(frames[-1].image.size, (32, 16))
        self.assertEqual(len({frame.image.tobytes() for frame in frames[:8]}), 1)
        for frame in frames:
            self.assertLessEqual(set(frame.image.get_flattened_data()), {(0, 0, 0, 0), (20, 18, 16, 255),
                                                          (154, 154, 154, 255), (110, 110, 110, 255)})

    def test_direction_scale_is_shared_within_each_four_pose_group(self) -> None:
        image = fixture()
        draw = ImageDraw.Draw(image)
        draw.rectangle((0, 0, 255, 63), fill='#FF00FF')
        for col in range(4):
            draw.rectangle((col * 64 + 24 + col, 14, col * 64 + 39 + col, 45), fill='#9A9A9A')
        transform = build_transform(image)
        self.assertNotEqual(transform.south, transform.east)
        frames = apply_transform(image, transform)
        restored = apply_transform(image, SheetTransform.from_json(transform.to_json()))
        self.assertEqual([frame.image.tobytes() for frame in frames],
                         [frame.image.tobytes() for frame in restored])
        bboxes = [frame.image.getbbox() for frame in frames[:8]]
        self.assertTrue(all(bbox is not None for bbox in bboxes))
        present = [bbox for bbox in bboxes if bbox is not None]
        self.assertTrue(all(27 <= bbox[3] - bbox[1] <= 28 for bbox in present))
        self.assertGreater(present[3][0], present[0][0])
        self.assertEqual(len({frame.image.tobytes() for frame in frames[4:8]}), 1)

    def test_layout_missing_extra_and_size_drift(self) -> None:
        image = fixture()
        transform = build_transform(image)
        missing = image.copy()
        ImageDraw.Draw(missing).rectangle((0, 0, 63, 63), fill='#FF00FF')
        with self.assertRaises(PipelineError):
            _ = build_transform(missing)
        extra = image.copy()
        extra.putpixel((200, 200), (154, 154, 154, 255))
        with self.assertRaises(PipelineError):
            _ = apply_transform(extra, transform)
        with self.assertRaises(PipelineError):
            _ = apply_transform(image.crop((0, 0, 255, 256)), transform)

    def test_identical_shaded_source_extracts_no_layer_under_same_palette(self) -> None:
        image = fixture()
        draw = ImageDraw.Draw(image)
        for row in range(2):
            for col in range(4):
                draw.rectangle((col * 64 + 25, row * 64 + 12,
                                col * 64 + 37, row * 64 + 24), fill='#AEAEAE')
        transform = build_transform(image)
        master = ((20, 18, 16), (154, 154, 154), (110, 110, 110), (176, 176, 176))
        body = apply_transform(image, transform)
        comparison = apply_transform(image, transform, master)
        self.assertNotEqual(body[0].image.tobytes(), comparison[0].image.tobytes())
        layers = extract_variants(image, image, transform, master)
        self.assertTrue(all(frame.image.getbbox() is None for frame in layers))
        self.assertTrue(all(set(frame.image.get_flattened_data()) <=
                            {(0, 0, 0, 0), (20, 18, 16, 255), (154, 154, 154, 255), (110, 110, 110, 255)}
                            for frame in body))

    def test_variant_margin_and_erasure_rejection(self) -> None:
        image = fixture()
        transform = build_transform(image)
        worn = image.copy()
        for row in range(2):
            for col in range(4):
                ImageDraw.Draw(worn).rectangle((col * 64 + 25, row * 64 + 8,
                                               col * 64 + 38, row * 64 + 14), fill='#B0B0B0')
        colors = ((20, 18, 16), (154, 154, 154), (110, 110, 110), (176, 176, 176))
        layers = extract_variants(image, worn, transform, colors)
        self.assertEqual(len(layers), 9)
        self.assertGreater(sum(frame.image.getbbox() is not None for frame in layers), 0)
        bad = worn.copy()
        ImageDraw.Draw(bad).rectangle((24, 20, 39, 25), fill='#FF00FF')
        with self.assertRaises(PipelineError):
            _ = extract_variants(image, bad, transform, colors)
        bad = worn.copy()
        bad.putpixel((0, 0), (176, 176, 176, 255))
        with self.assertRaises(PipelineError):
            _ = apply_transform(bad, transform, colors)

    def test_child_scale_and_external_evidence(self) -> None:
        image = fixture()
        transform = build_transform(image, upright_height=24)
        frames = apply_transform(image, transform)
        bbox = frames[0].image.getbbox()
        assert bbox is not None
        self.assertEqual(bbox[3] - bbox[1], 24)
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'input.png'
            image.save(source)
            save_frames(frames, transform, source, Path(directory) / 'result')
            payload = json_object(decode_json((Path(directory) / 'result/frames.json').read_text()))
            self.assertEqual(payload['accepted'], False)
            self.assertEqual(len(json_array(payload['frames'])), 9)
            self.assertEqual(len(json_string(payload['source_sha256'])), 64)
        for malformed in ('{}', '[]', 'null', '{"source_size": [true, 256]}'):
            with self.subTest(malformed=malformed), self.assertRaises(PipelineError):
                _ = SheetTransform.from_json(malformed)
        invalid = json_object(decode_json(transform.to_json()))
        invalid['source_size'] = [True, 256]
        with self.assertRaises(PipelineError):
            _ = SheetTransform.from_json(json.dumps(invalid))


if __name__ == '__main__':
    _ = unittest.main()
