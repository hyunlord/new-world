import unittest

from PIL import Image

from . import single_frame
from .models import PipelineError
from .outline import enforce_outline

PALETTE = ((20, 18, 16), (154, 154, 154), (176, 176, 176))


def mannequin(height: int = 32) -> Image.Image:
    image = Image.new('RGBA', (32, height))
    image.paste((154, 154, 154, 255), (10, 5, 22, min(height - 2, 26)))
    return enforce_outline(image)


class SingleFrameTests(unittest.TestCase):
    def test_identical_reference_has_empty_layer(self) -> None:
        body = mannequin()
        reference = single_frame.prepare_reference(body)
        self.assertEqual(reference.size, (1024, 1024))
        result = single_frame.process_single_variant(body, reference, PALETTE)
        self.assertIsNone(result.layer.image.getbbox())
        self.assertEqual(result.changed_pixels, 0)
        self.assertEqual(result.source_size, (1024, 1024))

    def test_noninteger_generated_size_keeps_coordinates(self) -> None:
        body = mannequin()
        raw = single_frame.prepare_reference(body).resize((1254, 1254), Image.Resampling.NEAREST)
        result = single_frame.process_single_variant(body, raw, PALETTE)
        self.assertEqual(result.variant.tobytes(), body.tobytes())
        self.assertEqual(result.source_size, (1254, 1254))
        self.assertEqual(result.reduced_changed_pixels, 0)

    def test_hat_is_extracted_with_body_context(self) -> None:
        body = mannequin()
        variant = body.copy()
        variant.paste((176, 176, 176, 255), (10, 2, 22, 8))
        raw = single_frame.prepare_reference(variant)
        result = single_frame.process_single_variant(body, raw, PALETTE, (9, 1, 23, 9))
        self.assertGreater(result.changed_pixels, 0)
        self.assertGreater(result.layer.difference_pixels, 0)
        self.assertEqual(result.outline_context.tobytes(), body.tobytes())
        self.assertEqual(body.getpixel((15, 10)), (154, 154, 154, 255))

    def test_erase_and_roi_drift_rejected(self) -> None:
        body = mannequin()
        erased = body.copy()
        erased.putpixel((15, 15), (0, 0, 0, 0))
        with self.assertRaises(PipelineError):
            _ = single_frame.process_single_variant(body, single_frame.prepare_reference(erased), PALETTE)
        altered = body.copy()
        altered.putpixel((15, 15), (176, 176, 176, 255))
        with self.assertRaises(PipelineError):
            _ = single_frame.process_single_variant(body, single_frame.prepare_reference(altered), PALETTE, (0, 0, 32, 10))

    def test_sleep_preserves_origin_rejects_bottom_foreground(self) -> None:
        body = mannequin(16)
        reference = single_frame.prepare_reference(body)
        result = single_frame.process_single_variant(body, reference, PALETTE)
        self.assertEqual(result.variant.size, (32, 16))
        self.assertEqual(result.base.tobytes(), body.tobytes())
        reference.putpixel((1000, 1000), (154, 154, 154, 255))
        with self.assertRaises(PipelineError):
            _ = single_frame.process_single_variant(body, reference, PALETTE)

    def test_non_square_and_invalid_roi_rejected(self) -> None:
        body = mannequin()
        with self.assertRaises(PipelineError):
            _ = single_frame.process_single_variant(body, Image.new('RGB', (64, 32)), PALETTE)
        with self.assertRaises(PipelineError):
            _ = single_frame.process_single_variant(body, single_frame.prepare_reference(body), PALETTE, (-1, 0, 32, 32))
