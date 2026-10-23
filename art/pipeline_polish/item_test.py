# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: PYTHONPATH=. python -m pytest art/pipeline_polish/item_test.py
"""Regression checks for frozen review packets and explicit repair selection."""

from __future__ import annotations

import sys
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

import pytest

from art.pipeline_env1.items_records import JsonRecord
from art.pipeline_polish import item_build, item_evidence


def test_final_repair_overrides_earlier_numeric_pass(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    records = [
        JsonRecord(
            {
                "id": name,
                "kind": "asset",
                "status": "COMPLETED",
                "attempt": 1,
                "numeric": {"numeric_pass": True},
                "normalized": "first.png",
                "normalized_sha256": "first",
                "raw": "first_raw.png",
                "raw_sha256": "first",
            }
        )
        for name in item_build.REGENERATE
    ]
    records.append(
        JsonRecord(
            {
                **records[0].fields,
                "id": "firewood",
                "attempt": 2,
                "numeric": {"numeric_pass": False},
                "normalized": "repair.png",
            }
        )
    )
    monkeypatch.setattr(item_build, "DATA", tmp_path)
    item_build.save_json(tmp_path / "selection_round1.json", {"firewood": 2})
    selected = {r.text("id"): r for r in item_build.selected_assets(records, 1)}
    assert selected["firewood"].text("path") == "repair.png"
    assert selected["firewood"].flag("numeric_pass") is False
    assert selected["spear"].flag("reused") is True


def test_dispatched_packet_refuses_rebuild(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    packet = tmp_path / "round1"
    packet.mkdir()
    sentinel = packet / "packet_manifest.json"
    _ = sentinel.write_text("frozen")
    monkeypatch.setattr(item_build, "DATA", tmp_path)
    monkeypatch.setattr(sys, "argv", ["item_build", "1"])
    with pytest.raises(FileExistsError, match="already frozen"):
        item_build.main()
    assert sentinel.read_text() == "frozen"


@pytest.mark.parametrize("changed", ["board.png", "answer_key.json", "sprite.png"])
def test_packet_audit_rejects_changed_bytes(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    changed: str,
) -> None:
    for name in ("board.png", "sprite.png"):
        _ = (tmp_path / name).write_bytes(b"original")
    item_build.save_json(
        tmp_path / "answer_key.json", [{"path": "sprite.png", "sha256": item_build.digest(tmp_path / "sprite.png")}]
    )
    item_build.save_json(
        tmp_path / "packet_manifest.json",
        {
            "answer_key_sha256": item_build.digest(tmp_path / "answer_key.json"),
            "boards": [{"path": str(tmp_path / "board.png"), "sha256": item_build.digest(tmp_path / "board.png")}],
        },
    )
    monkeypatch.setattr(item_evidence, "ROOT", tmp_path)
    _ = item_evidence.verify_packet(tmp_path)
    _ = (tmp_path / changed).write_bytes(b"changed")
    with pytest.raises(RuntimeError, match="hash mismatch"):
        _ = item_evidence.verify_packet(tmp_path)
