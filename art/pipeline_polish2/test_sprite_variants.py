"""Mechanical tint guarantees; these do not replace semantic mask acceptance."""
from __future__ import annotations

import hashlib
from pathlib import Path

import numpy as np
import pytest
from PIL import Image
from pydantic import BaseModel

from art.compositor.face_variants import ROOT, FacePlan
from art.pipeline_polish2.sprite_annotations import AnatomyAnnotations
from art.pipeline_polish2.sprite_colour import lab_to_rgb, rgb_to_lab, shifted
from art.pipeline_polish2.sprite_freeze import FreezeGuard
from art.pipeline_polish2.sprite_landmarks import FrameGuide
from art.pipeline_polish2.sprite_masks import detect
from art.pipeline_polish2.sprite_regions import (
    SourceEnvelope,
    facial_hair_support,
    polygon_union,
    semantic_hair_support,
    source_hair_reference,
)
from art.pipeline_polish2.sprite_sources import load_envelopes


class OriginalManifest(BaseModel):
    files: dict[str, str]


class Transform(BaseModel):
    look: str
    variant: str
    skin_delta_lab: tuple[float, float, float]
    hair_delta_lab: tuple[float, float, float]


class TransformManifest(BaseModel):
    transforms: tuple[Transform, ...]


def test_lab_roundtrip_and_selective_shift() -> None:
    colours = np.array(((0, 0, 0), (255, 255, 255), (113, 74, 47), (160, 155, 140)), dtype=np.float64)
    assert np.array_equal(lab_to_rgb(rgb_to_lab(colours)), colours.astype(np.uint8))
    pixels = np.array([[(100, 70, 40, 128), (80, 50, 30, 0)]], dtype=np.uint8)
    mask = np.array([[True, False]])
    result = np.asarray(shifted(Image.fromarray(pixels), mask, (15, 5, -2)))
    assert np.array_equal(result[:, :, 3], pixels[:, :, 3])
    assert np.array_equal(result[~mask], pixels[~mask])
    assert not np.array_equal(result[mask, :3], pixels[mask, :3])
    with pytest.raises(ValueError, match="dimensions"):
        _ = shifted(Image.fromarray(pixels), np.zeros((3, 3), dtype=np.bool_), (0, 0, 0))


def test_all_originals_match_their_accepted_manifests() -> None:
    for look in FacePlan.read().looks:
        folder = ROOT / "art/character/route_i" / look.look
        manifest = OriginalManifest.model_validate_json((folder / "MANIFEST.json").read_bytes())
        for name, digest in manifest.files.items():
            assert hashlib.sha256((folder / name).read_bytes()).hexdigest() == digest, (look.look, name)


def test_masks_repeat_and_include_protected_ink_in_hair_reference() -> None:
    look = FacePlan.read().looks[0].look
    annotations = AnatomyAnnotations.read(ROOT / "art/character/variant_anatomy.json")
    with Image.open(ROOT / "art/character/route_i" / look / "S/idle.png") as opened:
        source = opened.convert("RGBA")
    guide = FrameGuide(False, False, annotation=annotations.for_look(look))
    first, second = detect(source, guide), detect(source, guide)
    assert np.array_equal(first.skin, second.skin)
    assert np.array_equal(first.hair, second.hair)
    assert not np.any((first.skin | first.hair) & first.outline)
    assert np.any(first.hair_silhouette & first.outline)


def test_every_baked_frame_preserves_alpha_and_all_unmasked_bytes() -> None:
    manifest = TransformManifest.model_validate_json((ROOT / "art/character/variant_transforms.json").read_bytes())
    transforms = {(entry.look, entry.variant): entry for entry in manifest.transforms}
    assert len(transforms) == 204
    checked = 0
    for look in FacePlan.read().looks:
        folder = ROOT / "art/character/route_i" / look.look
        for facing in ("S", "E"):
            for pose in ("idle", "walk_a", "work_a", "carry", "sleep"):
                with Image.open(folder / facing / f"{pose}.png") as opened:
                    source = np.asarray(opened.convert("RGBA"))
                mask_folder = ROOT / "art/character/variant_masks" / look.look / facing
                with Image.open(mask_folder / f"{pose}_skin.png") as opened:
                    skin = np.asarray(opened) > 0
                with Image.open(mask_folder / f"{pose}_hair.png") as opened:
                    hair = np.asarray(opened) > 0
                assert not np.any(skin & hair)
                outline = (rgb_to_lab(source[:, :, :3].astype(np.float64))[:, :, 0] <= 18) | (source[:, :, 3] < 128)
                assert not np.any((skin | hair) & outline)
                for variant in look.variants:
                    with Image.open(folder / variant.id / facing / f"{pose}.png") as opened:
                        baked = np.asarray(opened.convert("RGBA"))
                    assert np.array_equal(source[:, :, 3], baked[:, :, 3])
                    assert np.array_equal(source[~(skin | hair)], baked[~(skin | hair)])
                    transform = transforms[(look.look, variant.id)]
                    reproduced = shifted(shifted(Image.fromarray(source), skin, transform.skin_delta_lab), hair, transform.hair_delta_lab)
                    assert np.array_equal(np.asarray(reproduced), baked)
                    checked += 1
    assert checked == 2040


def test_frozen_reference_guard_rejects_drift(tmp_path: Path) -> None:
    assert FreezeGuard.read().hashes
    path = tmp_path / "reference.png"
    original = Image.new("RGBA", (2, 2), (100, 70, 40, 255))
    original.save(path)
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    guard = FreezeGuard({path: digest})
    with pytest.raises(ValueError, match="Refusing to change"):
        guard.save_png(Image.new("RGBA", (2, 2), (101, 70, 40, 255)), path)
    assert hashlib.sha256(path.read_bytes()).hexdigest() == digest


def test_declared_neck_skin_cannot_become_hair_from_base_brown_cluster() -> None:
    look = "adult_F_fur_cloak_bun"
    path = ROOT / "art/character/route_i" / look / "S/idle.png"
    with Image.open(path) as opened:
        source = opened.convert("RGBA")
    annotations = AnatomyAnnotations.read(ROOT / "art/character/variant_anatomy.json")
    envelope = SourceEnvelope(look=look, source_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                              garment_exclusions=(), hair_envelopes=(((30, 45), (50, 45), (50, 70), (30, 70)),))
    control = detect(source, FrameGuide(False, False, annotation=annotations.for_look(look), envelope=envelope))
    assert control.hair[62, 40]
    protected = envelope.model_copy(update={"skin_supports": (((39, 61), (41, 61), (41, 63), (39, 63)),)})
    result = detect(source, FrameGuide(False, False, annotation=annotations.for_look(look), envelope=protected, refine_skin=True, refine_hair=True))
    assert not result.hair[62, 40]
    assert not result.skin[62, 40]


def test_observed_facial_hair_is_explicit_and_clothing_still_wins() -> None:
    square = (((0, 0), (3, 0), (3, 3), (0, 3)),)
    envelope = SourceEnvelope(look="test", source_sha256="0" * 64, garment_exclusions=(),
                              hair_envelopes=square, skin_supports=square, facial_hair_envelopes=(((1, 1), (2, 1), (2, 2), (1, 2)),))
    mask = semantic_hair_support(envelope, (4, 4))
    assert np.count_nonzero(mask) == 4
    blocked = envelope.model_copy(update={"garment_exclusions": square})
    assert not np.any(semantic_hair_support(blocked, (4, 4)))


def test_source_ear_hole_overrides_both_beard_classifiers() -> None:
    square = (((0, 0), (3, 0), (3, 3), (0, 3)),)
    ear = (((1, 1), (2, 1), (2, 2), (1, 2)),)
    envelope = SourceEnvelope(look="test", source_sha256="0" * 64, garment_exclusions=(),
                              hair_envelopes=(), skin_supports=square,
                              facial_hair_envelopes=square, hair_exclusions=ear)
    assert np.count_nonzero(facial_hair_support(envelope, (4, 4))) == 12
    assert not np.any(semantic_hair_support(envelope, (4, 4)) & polygon_union((4, 4), ear))


def test_all_source_frames_merge_without_dropping_geometry() -> None:
    index = load_envelopes()
    assert len(index.frames) == 340
    assert len(index.report_hashes) == 4
    assert len(index.frame_reports) == 340
    assert index.frames[("adult_F_fur_cloak_long_tied", "S", "carry")].prop_exclusions
    assert index.frames[("adult_M_fur_cloak_short", "E", "idle")].facial_hair_envelopes
    assert not index.frames[("child_M_fur_cloak_short", "E", "idle")].facial_hair_envelopes


def test_source_wood_prop_exclusion_preserves_confusable_colours() -> None:
    look = "adult_F_fur_cloak_long_tied"
    envelope = load_envelopes().frames[(look, "S", "carry")]
    with Image.open(ROOT / "art/character/route_i" / look / "S/carry.png") as opened:
        source = opened.convert("RGBA")
    with Image.open(ROOT / "art/character/route_i" / look / "S/idle.png") as opened:
        idle = opened.convert("RGBA")
    control = detect(source, FrameGuide(False, False, "carry", idle,
                     envelope=envelope.model_copy(update={"prop_exclusions": ()}), refine_skin=True, refine_hair=True))
    result = detect(source, FrameGuide(False, False, "carry", idle,
                    envelope=envelope, refine_skin=True, refine_hair=True))
    prop = polygon_union(result.skin.shape, envelope.prop_exclusions)
    assert np.count_nonzero((control.skin | control.hair) & prop) > 0
    assert not np.any((result.skin | result.hair) & prop)
    tinted = shifted(shifted(source, result.skin, (20, -10, 5)), result.hair, (35, -5, 10))
    assert np.array_equal(np.asarray(tinted)[prop], np.asarray(source)[prop])


def test_source_face_highlight_recovery_does_not_expand_limb_trim() -> None:
    index = load_envelopes()
    annotations = AnatomyAnnotations.read(ROOT / "art/character/variant_anatomy.json")
    cases = {
        "adult_F_fur_cloak_long_tied": ((), ((41, 57),)),
        "adult_F_fur_cloak_bun": (((31, 69), (31, 70)), ()),
        "adult_M_fur_cloak_bun": (((31, 85), (30, 86)), ()),
        "child_F_fur_cloak_long_tied": (((34, 86), (35, 86)), ()),
    }
    for look, (trim, skin) in cases.items():
        with Image.open(ROOT / "art/character/route_i" / look / "S/idle.png") as opened:
            source = opened.convert("RGBA")
        masks = detect(source, FrameGuide(look.startswith("child_"), False,
                       annotation=annotations.for_look(look), envelope=index.frames[(look, "S", "idle")],
                       refine_skin=True, refine_hair=True))
        assert all(not masks.skin[y, x] for x, y in trim)
        assert all(masks.skin[y, x] for x, y in skin)


def test_independent_hair_reference_includes_visible_dark_ink() -> None:
    rgba = np.array([[(28, 18, 10, 64), (255, 213, 137, 255), (91, 58, 33, 255)]], dtype=np.uint8)
    envelope = SourceEnvelope(look="test", source_sha256="0" * 64, garment_exclusions=(),
                              hair_envelopes=(((0, 0), (2, 0), (2, 1), (0, 1)),))
    reference = source_hair_reference(rgba, rgb_to_lab(rgba[:, :, :3].astype(np.float64)), envelope)
    assert reference.tolist() == [[True, False, True]]
