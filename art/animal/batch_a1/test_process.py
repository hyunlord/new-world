# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy", "scipy", "pydantic"]
# ///
# How to run: python3 -m pytest art/animal/batch_a1/test_process.py
"""Exercise the two new failure boundaries without inventing new animal art."""
from pathlib import Path

import numpy as np
import pytest
from PIL import Image

from art.animal.batch_a1 import check, process
from art.animal.batch_a1.check import inspect_frame, split_attack


def test_attack_pair_preserves_every_rgba_pixel() -> None:
    # Given two separated figures and transparent pixels with nonzero RGB.
    source = Image.new("RGBA", (100, 60), (7, 8, 9, 0))
    source.paste((30, 40, 50, 255), (10, 10, 30, 50))
    source.paste((60, 70, 80, 255), (60, 10, 80, 50))
    source.putpixel((49, 25), (1, 2, 3, 1))
    # When split at the empty center gutter.
    panels, _ = split_attack(source)
    # Then reconstruction is byte-identical, including transparent RGB.
    assert np.array_equal(np.concatenate([np.asarray(p) for p in panels], axis=1), np.asarray(source))


def test_attack_pair_rejects_visible_alpha_at_cut() -> None:
    # Given one visible pixel crossing the proposed crop boundary.
    source = Image.new("RGBA", (100, 60))
    source.putpixel((49, 25), (1, 2, 3, 16))
    # When splitting, then it must fail rather than clip that pixel.
    with pytest.raises(ValueError, match="gutter"):
        _ = split_attack(source)


def test_disconnected_opaque_fragment_fails() -> None:
    # Given an anchored figure with a detached opaque speck.
    source = Image.new("RGBA", (96, 80))
    source.paste((30, 40, 50, 255), (30, 30, 60, 73))
    source.putpixel((70, 20), (1, 2, 3, 128))
    # When checked, then connectivity must fail despite a correct anchor.
    result = inspect_frame(source, False, 56)
    assert result.contact and not result.connectivity


def test_empty_sprite_cannot_pass_alpha_or_anchor() -> None:
    # Given a blank canvas of the expected size.
    source = Image.new("RGBA", (96, 80))
    # When checked, then a valid canvas is insufficient to pass.
    result = inspect_frame(source, False, 56)
    assert result.canvas and not result.alpha and not result.contact


@pytest.fixture
def attack_manifest(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """Create a complete synthetic pair in an isolated output root."""
    root = tmp_path / "repo"
    monkeypatch.setattr(check, "ROOT", root)
    monkeypatch.setattr(process, "ROOT", root)
    source = Image.new("RGBA", (100, 60))
    source.paste((30, 40, 50, 255), (10, 10, 30, 50))
    source.paste((60, 70, 80, 255), (60, 10, 80, 50))
    raw = tmp_path / "source.png"
    source.save(raw)
    for species, special in (("deer", "graze"), ("wolf", "attack")):
        for facing in ("S", "E"):
            folder = root / "art/animal/polish" / species / "try2" / facing
            folder.mkdir(parents=True)
            for pose in ("stand", "walk", "run", special, "dead"):
                source.save(folder / f"{pose}.png")
    prompt = tmp_path / "prompt.txt"
    _ = prompt.write_text("Synthetic geometry fixture, not candidate art.\n")
    record = tmp_path / "generation.json"
    _ = record.write_text('{"generation": "original"}\n')
    spec = check.Spec(
        species="fixture", attempt=1, raw=raw, prompt=prompt,
        shoulder_y=10, hoof_y=50, shoulder_target=40, dead_length=56,
        poses=("attack",), references=(raw,), generator_record=record,
    )
    spec_path = tmp_path / "spec.json"
    _ = spec_path.write_text(spec.model_dump_json())
    manifest = process.process(spec_path)
    assert check.verify(manifest)
    return manifest


def test_generator_record_change_fails_verification(attack_manifest: Path) -> None:
    # Given a valid manifest whose generator record matched at creation.
    document = check.Manifest.model_validate_json(attack_manifest.read_text())
    record = check.Spec.model_validate(document.spec).generator_record
    assert record is not None
    # When its recorded evidence changes, its original digest becomes stale.
    _ = record.write_text('{"generation": "changed"}\n')
    # Then verification must reject the changed provenance.
    assert not check.verify(attack_manifest)


def test_generator_record_is_optional(attack_manifest: Path) -> None:
    # Given a valid manifest from a producer without generator evidence.
    document = check.Manifest.model_validate_json(attack_manifest.read_text())
    _ = attack_manifest.write_text(document.model_dump_json(exclude={"generator_record"}))
    # When checked, then absence of this optional record remains supported.
    assert check.verify(attack_manifest)
