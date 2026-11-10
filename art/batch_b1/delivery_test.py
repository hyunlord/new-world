# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "numpy", "scipy", "pytest", "typer"]
# ///
# How to run: python3 -m pytest art/batch_b1/delivery_test.py
"""Delivery regressions: no partial copies, overwrites, or index drift."""

from pathlib import Path

import pytest

from art.batch_b1.delivery_assets import plan_copies
from art.batch_b1.delivery_models import CopyPair, Grid


def test_missing_source_leaves_destination_unwritten(tmp_path: Path) -> None:
    # Given a valid first source and a missing second source.
    source = tmp_path / "source"
    _ = source.write_bytes(b"source bytes")
    destination = tmp_path / "destination"
    pairs = (
        CopyPair(source=source, destination=destination),
        CopyPair(source=tmp_path / "missing", destination=tmp_path / "second"),
    )
    # When the complete transfer is preflighted.
    with pytest.raises(FileNotFoundError):
        _ = plan_copies(pairs)
    # Then even the valid first destination is untouched.
    assert not destination.exists()


def test_conflicting_destination_is_never_overwritten(tmp_path: Path) -> None:
    # Given different source and destination bytes.
    source, destination = tmp_path / "source", tmp_path / "destination"
    _ = source.write_bytes(b"new bytes")
    _ = destination.write_bytes(b"owned existing bytes")
    # When a copy is planned.
    with pytest.raises(FileExistsError):
        _ = plan_copies((CopyPair(source=source, destination=destination),))
    # Then the existing file remains exact.
    assert destination.read_bytes() == b"owned existing bytes"


def test_identical_destination_is_idempotent(tmp_path: Path) -> None:
    # Given a previously assembled byte-identical file.
    source, destination = tmp_path / "source", tmp_path / "destination"
    _ = source.write_bytes(b"same")
    _ = destination.write_bytes(b"same")
    # When the same transfer is planned.
    result = plan_copies((CopyPair(source=source, destination=destination),))
    # Then the row records reuse rather than a replacement.
    assert result[0].existed


def test_grid_indices_keep_top_inside_sex_and_age() -> None:
    # Given a new top appended to the old top order.
    grid = Grid(
        ages=("child", "adult"),
        sexes=("M", "F"),
        tops=("hide", "linen"),
        hairs=("short", "bun"),
        fallbacks={},
    )
    # When bridge look IDs are enumerated.
    result = grid.logical_looks()
    # Then their indices follow the actual bridge nesting, not alphabetic order.
    assert result[0:5] == (
        "child_M_hide_short",
        "child_M_hide_bun",
        "child_M_linen_short",
        "child_M_linen_bun",
        "child_F_hide_short",
    )
    assert result[8] == "adult_M_hide_short"


def delivery_fixture(tmp_path: Path) -> tuple[Path, Path]:
    """Use the real catalog/identity contract with a tiny pinned-config fixture."""
    import json
    import shutil

    from art.batch_b1.delivery_models import Catalog
    from art.batch_b1.models import REPO

    root, evidence = tmp_path / "repo", tmp_path / "evidence"
    (root / "art/batch_b1").mkdir(parents=True)
    (root / "art/character").mkdir(parents=True)
    (evidence / "records").mkdir(parents=True)
    _ = shutil.copyfile(
        REPO / "art/batch_b1/CATALOG.json", root / "art/batch_b1/CATALOG.json"
    )
    _ = shutil.copyfile(
        REPO / "art/character/FACE_VARIANTS.json",
        root / "art/character/FACE_VARIANTS.json",
    )
    catalog = Catalog.model_validate_json(
        (root / "art/batch_b1/CATALOG.json").read_bytes()
    )
    new = {look.id for look in catalog.looks}
    grid = catalog.proposed_look_grid.model_dump(mode="json")
    grid["tops"] = ["loincloth", "hide_wrap", "fur_cloak"]
    config = {
        "people": {
            "looks": [
                look
                for look in catalog.proposed_look_grid.logical_looks()
                if look not in new and look not in catalog.proposed_look_grid.fallbacks
            ],
            "data": [{"source": "keep/baseline.json"}],
            "sentinel": "unchanged",
        },
        "look_grid": grid,
        "other_family": {"sentinel": "untouched"},
    }
    _ = (evidence / "records/reference_art_import.json").write_text(json.dumps(config))
    _ = (evidence / "records/reference_bindings.json").write_text(
        json.dumps({"people": {"pose_for_state": {"work": "work_a"}}})
    )
    return root, evidence


def test_missing_new_look_blocks_all_baseline_copies(tmp_path: Path) -> None:
    from art.batch_b1.delivery import assemble

    root, evidence = delivery_fixture(tmp_path)
    baseline = root / "art/character/route_i/child_M_loincloth_short/S/idle.png"
    baseline.parent.mkdir(parents=True)
    _ = baseline.write_bytes(b"original")
    with pytest.raises(FileNotFoundError):
        _ = assemble(root, evidence)
    assert not (root / "art/character/batch_b1").exists()
    assert not (root / "art/batch_b1/drafts").exists()
    assert not (evidence / "records/delivery_inventory.json").exists()


def test_drafts_preserve_identity_axes_and_old_data_paths(tmp_path: Path) -> None:
    from art.batch_b1.delivery_drafts import draft_payloads, object_value, strings
    from art.batch_b1.delivery_models import Catalog, JsonDocument
    from art.compositor.face_variants import FacePlan

    root, evidence = delivery_fixture(tmp_path)
    payloads = draft_payloads(root, evidence)
    directory = root / "art/batch_b1/drafts"
    fragment = JsonDocument.model_validate_json(
        payloads[directory / "people.fragment.json"]
    ).root
    people = object_value(fragment["people"])
    assert set(fragment) == {"people", "look_grid"}
    assert people["data"] == [{"source": "keep/baseline.json"}]
    assert people["sentinel"] == "unchanged"
    assert len(strings(people["looks"])) == 118
    baseline = FacePlan.read(root / "art/character/FACE_VARIANTS.json")
    proposed = FacePlan.model_validate_json(payloads[directory / "FACE_VARIANTS.json"])
    catalog = Catalog.model_validate_json(
        (root / "art/batch_b1/CATALOG.json").read_bytes()
    )
    assert len(proposed.looks) == 118
    for look in catalog.looks:
        assert (
            proposed.for_look(look.id).variants
            == baseline.for_look(
                f"{look.age}_{look.sex}_hide_wrap_{look.hair}"
            ).variants
        )
    assert (
        b"top-code mapping remain pending"
        in payloads[directory / "APPEARANCE_ORDER.md"]
    )


def test_output_conflict_prevents_all_writes(tmp_path: Path) -> None:
    from art.batch_b1.delivery import write_preflighted

    first, conflict = tmp_path / "first", tmp_path / "owned"
    _ = conflict.write_bytes(b"owned")
    with pytest.raises(FileExistsError):
        write_preflighted({first: b"new", conflict: b"different"})
    assert not first.exists()
    assert conflict.read_bytes() == b"owned"


def test_duplicate_copy_destination_is_rejected(tmp_path: Path) -> None:
    source = tmp_path / "source"
    _ = source.write_bytes(b"source")
    pair = CopyPair(source=source, destination=tmp_path / "dest")
    with pytest.raises(ValueError, match="duplicate destination"):
        _ = plan_copies((pair, pair))


def test_draft_cli_accepts_evidence_option(tmp_path: Path) -> None:
    from typer.testing import CliRunner

    from art.batch_b1.delivery import app
    from art.batch_b1.delivery_drafts import object_value
    from art.batch_b1.delivery_models import JsonDocument

    root, evidence = delivery_fixture(tmp_path)
    result = CliRunner().invoke(
        app, ["draft", "--evidence", str(evidence), "--root", str(root)]
    )
    assert result.exit_code == 0, result.output
    directory = root / "art/batch_b1/drafts"
    proposal = JsonDocument.model_validate_json(
        (directory / "bindings.proposal.json").read_bytes()
    ).root
    tops = object_value(proposal["top_code_mapping_proposal"])
    assert set(object_value(tops["current_bridge_top_codes"])) == {
        str(code) for code in range(7)
    }
    assert proposal["status"] == "PROPOSAL_NOT_RUNTIME_ACTIVATION"
    assert (
        b"Counts are conditional on successful full assembly"
        in (directory / "APPEARANCE_ORDER.md").read_bytes()
    )
    repeated = CliRunner().invoke(
        app, ["draft", "--evidence", str(evidence), "--root", str(root)]
    )
    assert repeated.exit_code == 0, repeated.output
