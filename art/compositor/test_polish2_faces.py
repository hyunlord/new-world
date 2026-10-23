# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "pydantic", "numpy"]
# ///
# How to run: python3 -m pytest art/compositor/test_polish2_faces.py -q.
"""Exercise both actual file-size outcomes without touching final portrait art."""
import subprocess
from pathlib import Path

import numpy as np
import pytest
from PIL import Image

from art.compositor import polish2_faces as faces
from art.compositor.face_variants import FacePlan
from art.compositor.render import sha256


def fixture_sources(root: Path, *, noise: bool) -> FacePlan:
    """Make deterministic native-size inputs for all frozen owner slots."""
    plan = FacePlan.read()
    rng = np.random.default_rng(316)
    for name in plan.owner_sample:
        look = plan.for_look(name)
        for variant in look.variants:
            portrait = root / "art/portrait/polish2" / look.look / f"{variant.id}.png"
            sprite = root / "art/character/route_i" / look.look / variant.id / "S/idle.png"
            portrait.parent.mkdir(parents=True, exist_ok=True)
            sprite.parent.mkdir(parents=True, exist_ok=True)
            pixels = rng.integers(0, 256, (128, 128, 3), dtype=np.uint8)
            image = Image.fromarray(pixels) if noise else Image.new("RGB", (128, 128), (17, 44, 201))
            image.save(portrait)
            Image.new("RGBA", (80, 112), (110, 80, 32, 255)).save(sprite)
    return plan


def test_small_full_sheet_keeps_all_48_in_the_original_filename(tmp_path: Path) -> None:
    # Given all48 native portraits and frames that fit as one compressed sheet.
    plan = fixture_sources(tmp_path, noise=False)
    destination = tmp_path / "art/boards"
    destination.mkdir(parents=True)
    _ = (destination / "faces_owner.children.png").write_bytes(b"obsolete split output")
    _ = (destination / "faces_owner.children.json").write_text("{}")
    # When producing the owner-board set from those exact inputs.
    index = faces.faces_board(plan, root=tmp_path, external=tmp_path / "raw")
    # Then the original filename retains all48 pairs and binds the complete source.
    assert index.layout == "single"
    assert len(index.outputs) == 1 and len(index.members) == 48
    assert index.outputs[0].path == "art/boards/faces_owner.png"
    assert len(index.outputs[0].panels) == 96
    assert index.full_sheet.sha256 == index.outputs[0].sha256
    assert not (tmp_path / "art/boards/faces_owner.children.png").exists()
    assert not (tmp_path / "art/boards/faces_owner.children.json").exists()
    assert (tmp_path / "art/boards/faces_owner.index.json").is_file()
    parsed = faces.FaceBoardIndex.model_validate_json((tmp_path / "art/boards/faces_owner.index.json").read_bytes())
    assert parsed == index


def test_large_full_sheet_splits_only_after_lossless_attempt_and_retains_all_pixels(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch,
) -> None:
    # Given48 independent high-entropy portraits whose complete PNG exceeds1.5MB.
    plan = fixture_sources(tmp_path, noise=True)
    invoked: list[list[str]] = []
    original_run = subprocess.run

    def observed_run(args: list[str], *, check: bool, capture_output: bool,
                     timeout: int) -> subprocess.CompletedProcess[bytes]:
        invoked.append(args)
        return original_run(args, check=check, capture_output=capture_output, timeout=timeout)

    monkeypatch.setattr(subprocess, "run", observed_run)
    # When the real encoder and installed pngcrush process the complete image.
    index = faces.faces_board(plan, root=tmp_path, external=tmp_path / "raw")
    # Then every original pair survives in one of two capped, exact-scale outputs.
    assert index.layout == "adult_child_split"
    assert index.full_sheet.bytes > 1_500_000
    assert len(index.outputs) == 2 and len(index.members) == 48
    assert [record.path for record in index.outputs] == [
        "art/boards/faces_owner.png", "art/boards/faces_owner.children.png"]
    assert all(record.bytes <= 1_500_000 and len(record.panels) == 48 for record in index.outputs)
    assert all(member.output_path.endswith("children.png") == member.look.startswith("child_")
               for member in index.members)
    assert sha256(tmp_path / index.full_sheet.path) == index.full_sheet.sha256
    assert len(invoked) == 1 and invoked[0][:4] == ["xcrun", "pngcrush", "-q", "-brute"]
    for member in index.members:
        output = next(record for record in index.outputs if record.path == member.output_path)
        for full_index, output_index in zip(member.full_panel_indices, member.output_panel_indices, strict=True):
            full_panel, output_panel = index.full_sheet.panels[full_index], output.panels[output_index]
            assert (full_panel.path, full_panel.sha256, full_panel.crop, full_panel.integer_scale) == (
                output_panel.path, output_panel.sha256, output_panel.crop, output_panel.integer_scale)
    for record in index.outputs:
        with Image.open(tmp_path / record.path) as board:
            for panel in record.panels:
                with Image.open(panel.path) as source:
                    expected = source.convert("RGB").crop(panel.crop)
                if panel.integer_scale == 2:
                    expected = expected.resize((160, 224), Image.Resampling.NEAREST)
                x, y = panel.origin
                assert board.crop((x, y, x + expected.width, y + expected.height)).tobytes() == expected.tobytes()


def test_owner_sample_must_have_four_adult_and_four_child_looks(tmp_path: Path) -> None:
    # Given a malformed frozen sample with only seven looks.
    plan = fixture_sources(tmp_path, noise=False)
    invalid = plan.model_copy(update={"owner_sample": plan.owner_sample[:-1]})
    # When attempting to produce a board that would omit a required look.
    with pytest.raises(ValueError, match="four adult and four child"):
        _ = faces.faces_board(invalid, root=tmp_path, external=tmp_path / "raw")
    # Then no misleading final board has been written.
    assert not (tmp_path / "art/boards/faces_owner.png").exists()


def test_portrait_dimensions_are_exact_not_silently_cropped(tmp_path: Path) -> None:
    # Given a wrong-size canonical portrait among the frozen48 inputs.
    plan = fixture_sources(tmp_path, noise=False)
    portrait = next((tmp_path / "art/portrait/polish2").rglob("*.png"))
    Image.new("RGB", (129, 128)).save(portrait)
    # When rendering the board, the boundary rejects the malformed asset.
    with pytest.raises(ValueError, match="128"):
        _ = faces.faces_board(plan, root=tmp_path, external=tmp_path / "raw")
    # Then no source crop or final board masks the dimension problem.
    assert not (tmp_path / "art/boards/faces_owner.png").exists()
