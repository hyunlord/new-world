# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "numpy", "scipy", "scikit-image"]
# ///
# How to run: uv run --with pillow --with pydantic --with numpy --with scipy --with scikit-image python -m art.pipeline_polish.portrait_finalize
"""Bind regenerated and retained portrait evidence without treating style as a gate."""
from __future__ import annotations

import json
from dataclasses import asdict
from pathlib import Path
from typing import ClassVar, Final, TypedDict

from PIL import Image, ImageDraw, ImageOps
from pydantic import BaseModel, ConfigDict, JsonValue, TypeAdapter

from art.pipeline2.judge import judge, load_calibration
from art.pipeline_env1.portrait_finalize import InspectionFile, geometry
from art.pipeline_polish.portrait_prepare import DATA, ROOT, TARGETS, digest

DOCS: Final = ROOT / "docs/v8/artpolish"


class Attempt(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    attempt: int
    canonical_sha256: str
    numeric_pass: bool
    numeric_checks: dict[str, bool]
    model_scores: dict[str, JsonValue]
    structural: dict[str, JsonValue]


class Original(BaseModel):
    look: str
    selected_attempt: int
    canonical_path: Path
    attempts: list[Attempt]


class Baseline(BaseModel):
    records: list[Original]


class Generation(BaseModel):
    id: str
    kind: str
    attempt: int
    status: str
    raw: Path
    raw_sha256: str
    source: Path
    tool_sha256: str
    prompt: str
    references: list[dict[str, str]]
    started_at: str
    completed_at: str


class ModelScore(BaseModel):
    look: str
    attempt: int
    medium: int
    line: int
    light: int
    palette: int
    scorecard: Path
    scorecard_sha256: str
    method: str


class BlindSummary(BaseModel):
    reviewer: str
    correct: int
    total: int
    passed: bool


class BlindEvidence(BaseModel):
    answer_key: Path
    answer_key_sha256: str
    summaries: list[BlindSummary]
    passed: bool


class PortraitRecord(TypedDict):
    look: str
    path: str
    sha256: str
    sprite: str
    sprite_sha256: str
    source: str
    polish_art_attempts: int
    env1_selected_attempt: int
    env1_path: str
    env1_sha256: str
    structural: dict[str, JsonValue]
    numeric: dict[str, JsonValue]
    model_scores: dict[str, JsonValue]
    style_gates_acceptance: bool
    identity_test: str
    owner_adopted: bool
    status: str


def main() -> None:
    baseline = Baseline.model_validate_json((ROOT / "docs/v8/artenv1/portrait_manifest.json").read_text())
    generations = TypeAdapter(list[Generation]).validate_json((DATA / "generation_ledger.json").read_text())
    by_look = {row.id: row for row in generations if row.kind == "asset_generation"}
    observations = InspectionFile.model_validate_json((DATA / "inspections.json").read_text())
    inspections = {row.look: row for row in observations.inspections}
    model = TypeAdapter(list[ModelScore]).validate_json((DATA / "model_scores.json").read_text())
    model_by_look = {row.look: row for row in model}
    calibration = load_calibration(ROOT / "docs/v8/artenv1/portrait_calibration.json")
    blind_path = DOCS / "blind_portraits.json"
    blind = BlindEvidence.model_validate_json(blind_path.read_text())
    if digest(blind.answer_key) != blind.answer_key_sha256 or len(blind.summaries) != 2:
        message = "Blind evidence no longer matches the current packet"
        raise ValueError(message)
    identity_status = "PASS" if all(row.total == 34 and row.correct >= 30 for row in blind.summaries) else "FAIL"
    identity_note = identity_status + ": " + "; ".join(f"reviewer{row.reviewer} {row.correct}/{row.total}" for row in blind.summaries)
    records: list[PortraitRecord] = []
    diagnostics: list[dict[str, JsonValue]] = []
    overlay = Image.new("RGB", (6 * 160, 6 * 160), "#e5dcc8")
    draw = ImageDraw.Draw(overlay)
    for index, original in enumerate(sorted(baseline.records, key=lambda row: row.look)):
        previous = next(row for row in original.attempts if row.attempt == original.selected_attempt)
        identity = ROOT / "art/character/route_i" / original.look / "S/idle.png"
        path = ROOT / original.canonical_path
        structure = previous.structural
        numeric: dict[str, JsonValue] = {"numeric_pass": previous.numeric_pass, "numeric_checks": {key: value for key, value in previous.numeric_checks.items()},
                   "source": "Retained exact ENV1 evidence"}
        model_scores = previous.model_scores
        generation_count = 0
        if original.look in TARGETS:
            generation = by_look[original.look]
            if digest(generation.raw) != generation.raw_sha256 or digest(generation.source) != generation.tool_sha256:
                message = f"Generation hash mismatch: {original.look}"
                raise ValueError(message)
            path = ROOT / "art/portrait/polish" / f"{original.look}_try1.png"
            with Image.open(generation.raw) as raw:
                expected = ImageOps.contain(raw.convert("RGB"), (128, 128), Image.Resampling.LANCZOS)
            with Image.open(path) as canonical:
                if canonical.tobytes() != expected.tobytes():
                    message = f"Non-geometric pixel change: {original.look}"
                    raise ValueError(message)
            structure = geometry(inspections[original.look])
            verdict = judge(generation.raw, calibration, family="portrait")
            numeric = asdict(verdict)
            diagnostics.append({"look": original.look, "attempt": 1, "numeric": numeric})
            model_scores = model_by_look[original.look].model_dump(mode="json")
            generation_count = 1
        elif digest(path) != previous.canonical_sha256:
            message = f"Retained ENV1 bytes changed: {original.look}"
            raise ValueError(message)
        with Image.open(path) as image:
            if image.size != (128, 128):
                message = f"Wrong portrait dimensions: {path}"
                raise ValueError(message)
            x, y = index % 6 * 160 + 16, index // 6 * 160
            overlay.paste(image, (x, y))
        draw.ellipse((x + 20, y + 9, x + 108, y + 113), outline="cyan")
        for name, color in (("face_bbox", "lime"), ("head_bbox", "red")):
            box = TypeAdapter(tuple[int, int, int, int]).validate_python(structure[name])
            draw.rectangle((x + box[0], y + box[1], x + box[2], y + box[3]), outline=color)
        draw.text((x, y + 132), f"{index + 1:02d} {'POLISH' if generation_count else 'ENV1'}", fill="#33271d")
        records.append({"look": original.look, "path": str(path.relative_to(ROOT)), "sha256": digest(path),
                        "sprite": str(identity.relative_to(ROOT)), "sprite_sha256": digest(identity),
                        "source": "POLISH" if generation_count else "ENV1_RETAINED",
                        "polish_art_attempts": generation_count, "env1_selected_attempt": original.selected_attempt,
                        "env1_path": str(original.canonical_path), "env1_sha256": previous.canonical_sha256,
                        "structural": structure, "numeric": numeric, "model_scores": model_scores,
                        "style_gates_acceptance": False, "identity_test": identity_note,
                        "owner_adopted": False, "status": "OWNER_CHOICE_PENDING"})
    overlay.save(DATA / "structural_inspection_overlay.png", optimize=True)
    counts = {"portraits": len(records), "regenerated": len(by_look), "retained": len(records) - len(by_look),
              "art_calls": len(by_look), "scorecard_calls": len(generations) - len(by_look),
              "failed_calls": sum(row.status != "COMPLETED" for row in generations), "total_calls": len(generations),
              "structural_pass": sum(bool(row["structural"]["passed"]) for row in records),
              "numeric_pass_report_only": sum(bool(row["numeric"]["numeric_pass"]) for row in records),
              "owner_adopted": 0}
    board_path = ROOT / "art/boards/portraits_owner.png"
    if board_path.stat().st_size > 1_500_000:
        message = "Losslessly recompress the owner board before finalization"
        raise ValueError(message)
    payload = {"schema_version": 1, "instrument_change": "D1: numeric v2 and image-model style report only",
               "counts": counts, "board": "art/boards/portraits_owner.png", "records": records,
               "board_sha256": digest(board_path), "board_bytes": board_path.stat().st_size,
               "blind_packet": str(DATA / "blind/packet"), "identity_bar_per_reviewer": "30/34",
               "geometry_method": "ENV1 fixed ellipse center64,61 radii44,52; conservative manually observed head and face bboxes, approximately2px uncertainty; head margin>=2px",
               "generation_ledger": "docs/v8/artpolish/portrait_generation_ledger.json",
               "identity_evidence": {"path": str(blind_path.relative_to(ROOT)), "sha256": digest(blind_path),
                                     "summaries": [row.model_dump() for row in blind.summaries]},
               "owner_display": {"portrait_native": [128, 128], "portrait_small": [64, 64],
                                 "portrait_small_filter": "BILINEAR", "sprite_scale": 0.5,
                                 "sprite_filter": "LANCZOS", "blind_sprite_scale": 2,
                                 "blind_sprite_filter": "NEAREST"}}
    _ = (DOCS / "portrait_manifest.json").write_text(json.dumps(payload, indent=2) + "\n")
    _ = (DOCS / "portrait_numeric.json").write_text(json.dumps(diagnostics, indent=2) + "\n")
    _ = (DOCS / "portrait_inspections.json").write_text((DATA / "inspections.json").read_text())
    _ = (DOCS / "portrait_model_scores.json").write_text((DATA / "model_scores.json").read_text())
    _ = (DOCS / "portrait_generation_ledger.json").write_text((DATA / "generation_ledger.json").read_text())
    print(json.dumps(counts))


if __name__ == "__main__":
    main()
