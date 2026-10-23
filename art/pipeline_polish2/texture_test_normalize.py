# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m pytest art/pipeline_polish2/texture_test_normalize.py
"""Normalization keeps structural heights independent of alpha fringe."""

import json
from pathlib import Path
from typing import ClassVar

import pytest
from PIL import Image, ImageDraw
from pydantic import BaseModel, ConfigDict, TypeAdapter

from art.pipeline_polish2 import texture_normalize as pipeline
from art.pipeline_polish2.texture_normalize import TextureRequest, normalize


class AttemptEvidence(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    attempt: int
    numeric_pass: bool
    candidate_ready: bool


def test_opaque_door_height_ignores_translucent_fringe(tmp_path: Path) -> None:
    source = Image.new("RGBA", (200, 240), (0, 0, 0, 1))
    ImageDraw.Draw(source).rectangle((40, 30, 159, 209), fill=(160, 120, 80, 255))
    path = tmp_path / "door.png"
    source.save(path)
    request = TextureRequest(
        id="fixture/door",
        style="fixture",
        part="door",
        size=(64, 60),
        kind="door",
        axis="none",
        attempt=1,
        prompt="fixture",
        referenced_image_paths=(),
        transparent_background=True,
        source=path,
        generated_source=path,
    )
    image, _operations = normalize(request)
    bounds = image.getchannel("A").point([0] * 128 + [255] * 128).getbbox()
    assert bounds is not None
    assert bounds[3] - bounds[1] >= 58
    assert image.size == (64, 60)


def selection_fixture(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """Create real failed strips and a reviewed second-attempt selection."""
    root, data = tmp_path / "repo", tmp_path / "data"
    (root / "art/materials").mkdir(parents=True)
    data.mkdir()
    requests: list[TextureRequest] = []
    for attempt in (2, 3):
        source = Image.new("RGBA", (256, 64), (160, 120, 80, 255))
        ImageDraw.Draw(source).rectangle(
            (0, 0, 0, 63), fill=(attempt * 20, 40, 30, 255)
        )
        path = data / f"source{attempt}.png"
        source.save(path)
        requests.append(
            TextureRequest(
                id="fixture/top_ew",
                style="fixture",
                part="top_ew",
                size=(256, 64),
                kind="strip",
                axis="x",
                attempt=attempt,
                prompt="fixture",
                referenced_image_paths=(),
                transparent_background=False,
                source=path,
                generated_source=path,
            )
        )
    _ = (data / "completed.json").write_bytes(
        TypeAdapter(list[TextureRequest]).dump_json(requests)
    )
    _ = (root / "art/materials/texture_selections.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "selections": {
                    "fixture/top_ew": {
                        "attempt": 2,
                        "source_sha256": pipeline.digest(data / "source2.png"),
                        "reason": "Reviewed second attempt has the lower failed seam ratio.",
                        "review": {
                            "numeric_status": "FAIL",
                            "owner_approval": "NOT_CLAIMED",
                        },
                    }
                },
            }
        )
    )
    monkeypatch.setattr(pipeline, "ROOT", root)
    monkeypatch.setattr(pipeline, "DATA", data)
    return root


def test_reviewed_second_attempt_survives_full_normalization(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    root = selection_fixture(tmp_path, monkeypatch)

    pipeline.main()
    first_digest = pipeline.digest(root / "art/materials/textures/fixture/top_ew.png")
    pipeline.main()

    selected = AttemptEvidence.model_validate_json(
        (root / "art/materials/textures/fixture/top_ew.json").read_bytes()
    )
    assert selected.attempt == 2
    assert not selected.numeric_pass and not selected.candidate_ready
    assert (
        pipeline.digest(root / "art/materials/textures/fixture/top_ew.png")
        == first_digest
    )
    assert first_digest == pipeline.digest(
        tmp_path / "data/normalized/fixture_top_ew_try2.png"
    )
    evidence = TypeAdapter(list[AttemptEvidence]).validate_json(
        (tmp_path / "data/checks.json").read_bytes()
    )
    assert [row.attempt for row in evidence] == [2, 3]


def test_reviewed_selection_rejects_missing_attempt(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    root = selection_fixture(tmp_path, monkeypatch)
    path = root / "art/materials/texture_selections.json"
    rules = pipeline.TextureSelections.model_validate_json(path.read_bytes())
    choice = rules.selections["fixture/top_ew"].model_copy(update={"attempt": 1})
    updated = pipeline.TextureSelections(
        schema_version=1, selections={"fixture/top_ew": choice}
    )
    _ = path.write_text(updated.model_dump_json())

    with pytest.raises(ValueError, match="one existing attempt"):
        pipeline.main()

    assert not (root / "art/materials/textures").exists()


def test_reviewed_selection_rejects_changed_source_hash(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    root = selection_fixture(tmp_path, monkeypatch)
    path = root / "art/materials/texture_selections.json"
    rules = pipeline.TextureSelections.model_validate_json(path.read_bytes())
    choice = rules.selections["fixture/top_ew"].model_copy(
        update={"source_sha256": "0" * 64}
    )
    updated = pipeline.TextureSelections(
        schema_version=1, selections={"fixture/top_ew": choice}
    )
    _ = path.write_text(updated.model_dump_json())

    with pytest.raises(ValueError, match="source hash mismatch"):
        pipeline.main()

    assert not (root / "art/materials/textures").exists()


def test_reviewed_selection_rejects_visual_defect(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    root = selection_fixture(tmp_path, monkeypatch)
    path = tmp_path / "data/completed.json"
    adapter = TypeAdapter(list[TextureRequest])
    requests = adapter.validate_json(path.read_bytes())
    requests[0] = requests[0].model_copy(
        update={"visual_rejection": "White gutter violates full-bleed texture."}
    )
    _ = path.write_bytes(adapter.dump_json(requests))

    with pytest.raises(ValueError, match="visual rejection"):
        pipeline.main()

    assert not (root / "art/materials/textures").exists()
