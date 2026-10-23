"""Join immutable portrait evidence; failed controls keep every output a candidate."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import ClassVar

from PIL import Image, ImageDraw
from pydantic import BaseModel, ConfigDict, JsonValue, TypeAdapter

from art.pipeline2.judge import ModelScores, Verdict
from art.pipeline_env1.portrait_process import DATA, DOCS, ROOT, Generation


class Evidence(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)


class Numeric(Evidence):
    look: str
    attempt: int
    raw: Path
    raw_sha256: str
    canonical: Path
    canonical_sha256: str
    operations: list[str]
    numeric: Verdict


class Score(Evidence):
    id: str
    look: str
    attempt: int
    scorecard: str
    medium: int
    line: int
    light: int
    palette: int
    sentence: str

    def scores(self) -> ModelScores:
        return ModelScores(self.medium, self.line, self.light, self.palette, self.sentence)


class ScoreFile(Evidence):
    scores: list[Score]
    negative_controls: list[Score]


class Inspection(Evidence):
    look: str
    attempt: int
    head_bbox: tuple[int, int, int, int]
    face_bbox: tuple[int, int, int, int]
    head_uncropped: bool
    hair_match: bool
    top_match: bool
    extra_accessory: bool
    identity_note: str
    inspection_method: str


class InspectionFile(Evidence):
    inspections: list[Inspection]


class ReviewCall(Evidence):
    id: str
    kind: str
    source: Path
    path: Path
    prompt: str
    refs: list[Path]


class Prompt(Evidence):
    prompt: str
    referenced_image_paths: list[Path]
    transparent_background: bool


class Look(Evidence):
    look: str


class SourceManifest(Evidence):
    accepted_looks: list[Look]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def geometry(observation: Inspection) -> dict[str, JsonValue]:
    left, top, right, bottom = observation.face_bbox
    values = [((x - 64) / 44) ** 2 + ((y - 61) / 52) ** 2
              for x, y in ((left, top), (right, top), (right, bottom), (left, bottom))]
    head = observation.head_bbox
    margin = min(head[0], head[1], 128 - head[2], 128 - head[3])
    face_pass = max(values) <= 1
    return {"face_bbox": list(observation.face_bbox), "head_bbox": list(head),
            "face_bbox_corner_ellipse_max": max(values), "face_inside_fixed_oval": face_pass,
            "head_uncropped": observation.head_uncropped, "head_margin_px": margin,
            "head_margin_pass": margin >= 2,
            "passed": face_pass and observation.head_uncropped and margin >= 2,
            "method": observation.inspection_method}


def main() -> None:
    rows = TypeAdapter(list[Numeric]).validate_json((DOCS / "portrait_numeric.json").read_text())
    scores = ScoreFile.model_validate_json((DOCS / "portrait_model_scores.json").read_text())
    inspections = InspectionFile.model_validate_json((DOCS / "portrait_inspections.json").read_text())
    reviews = TypeAdapter(list[ReviewCall]).validate_json((DATA / "score_ledger.json").read_text())
    generations = TypeAdapter(list[Generation]).validate_json((DATA / "ledger.json").read_text())
    source = SourceManifest.model_validate_json((ROOT / "art/character/route_i/MANIFEST.json").read_text())
    score_map = {(row.look, row.attempt): row for row in scores.scores}
    inspection_map = {(row.look, row.attempt): row for row in inspections.inspections}
    review_map = {row.id: row for row in reviews}
    records: list[dict[str, JsonValue]] = []
    selected: list[tuple[Numeric, Score, Inspection]] = []
    for original in source.accepted_looks:
        attempts: list[JsonValue] = []
        choices = sorted((row for row in rows if row.look == original.look), key=lambda row: row.attempt)
        assert 1 <= len(choices) <= 2
        for row in choices:
            key = (row.look, row.attempt)
            score, observation = score_map[key], inspection_map[key]
            review = review_map[score.scorecard]
            prompt_path = DATA / f"{row.look}_try{row.attempt}.prompt.json"
            prompt = Prompt.model_validate_json(prompt_path.read_text())
            identity = ROOT / "art/character/route_i" / row.look / "S/idle.png"
            assert prompt.referenced_image_paths[0] == identity
            assert row.raw_sha256 == digest(row.raw)
            assert row.canonical_sha256 == digest(ROOT / row.canonical)
            with Image.open(ROOT / row.canonical) as image:
                assert image.size == (128, 128)
            structure = geometry(observation)
            identity_pass = observation.hair_match and observation.top_match and not observation.extra_accessory
            attempt_evidence = {"look": row.look, "attempt": row.attempt,
                "identity_reference": str(identity.relative_to(ROOT)), "identity_sha256": digest(identity),
                "raw_path": str(row.raw), "raw_sha256": row.raw_sha256,
                "canonical_path": str(row.canonical), "canonical_sha256": row.canonical_sha256,
                "file_bytes": (ROOT / row.canonical).stat().st_size, "operations": row.operations,
                "prompt_path": str(prompt_path), "prompt_sha256": digest(prompt_path),
                "references": [{"path": str(path), "sha256": digest(path)} for path in prompt.referenced_image_paths],
                "numeric_pass": row.numeric.numeric_pass, "numeric_checks": row.numeric.numeric_checks,
                "model_scores": score.model_dump(mode="json"), "model_pass": score.scores().passed,
                "scorecard_path": str(review.path), "scorecard_sha256": digest(review.path),
                "scorecard_input_sha256": digest(review.refs[-1]), "structural": structure,
                "identity": {"hair_match": observation.hair_match, "top_match": observation.top_match,
                    "extra_accessory": observation.extra_accessory, "passed": identity_pass,
                    "note": observation.identity_note, "facial_identity": "NOT_ESTABLISHED_FROM_LOW_RES_SPRITE"},
                "local_checks_pass": row.numeric.numeric_pass and score.scores().passed and bool(structure["passed"]) and identity_pass,
                "accepted": False, "status": "CANDIDATE_CALIBRATION_FAIL"}
            attempts.append(TypeAdapter[JsonValue](JsonValue).validate_python(attempt_evidence))
        chosen = choices[-1]
        selected.append((chosen, score_map[(chosen.look, chosen.attempt)], inspection_map[(chosen.look, chosen.attempt)]))
        records.append({"look": original.look, "selected_attempt": chosen.attempt,
                        "canonical_path": str(chosen.canonical), "try_count": len(choices), "attempts": attempts,
                        "accepted": False, "status": "CANDIDATE_CALIBRATION_FAIL"})
    counts = {"art_generations": len(generations), "model_scorecards": len(reviews), "failed_calls": 0,
              "generation_count": len(generations) + len(reviews), "adopted_look_count": len(selected),
              "attempt_numeric_pass": sum(row.numeric.numeric_pass for row in rows),
              "attempt_model_pass": sum(row.scores().passed for row in scores.scores),
              "selected_numeric_pass": sum(row.numeric.numeric_pass for row, _, _ in selected),
              "selected_model_pass": sum(score.scores().passed for _, score, _ in selected),
              "selected_structural_pass": sum(bool(geometry(obs)["passed"]) for _, _, obs in selected),
              "selected_identity_pass": sum(obs.hair_match and obs.top_match and not obs.extra_accessory for _, _, obs in selected),
              "accepted": 0}
    assert len(selected) == 34 and counts["generation_count"] <= 78
    board = Image.new("RGB", (6 * 224, 6 * 182 + 34), "#e5dcc8")
    draw = ImageDraw.Draw(board)
    draw.text((8, 8), "ENV1 PORTRAITS | 34 CANDIDATES | 128px + 64px | CALIBRATION FAIL", fill="#442a20")
    diagnostic = Image.new("RGB", (6 * 160, 6 * 156), "#e5dcc8")
    diagnostic_draw = ImageDraw.Draw(diagnostic)
    for index, (row, _, obs) in enumerate(selected):
        x, y = index % 6 * 224 + 8, index // 6 * 182 + 34
        with Image.open(ROOT / row.canonical) as image:
            board.paste(image, (x, y))
            board.paste(image.resize((64, 64), Image.Resampling.LANCZOS), (x + 136, y + 32))
            dx, dy = index % 6 * 160 + 16, index // 6 * 156
            diagnostic.paste(image, (dx, dy))
        draw.text((x, y + 132), f"{index + 1:02d} {row.look[:row.look.index('_', 6)]}", fill="#442a20")
        draw.text((x, y + 145), row.look.split('_', 2)[2], fill="#442a20")
        draw.text((x, y + 158), f"try {row.attempt} | candidate", fill="#7c3324")
        diagnostic_draw.ellipse((dx + 20, dy + 9, dx + 108, dy + 113), outline="cyan")
        for bounds, colour in ((obs.face_bbox, "lime"), (obs.head_bbox, "red")):
            left, top, right, bottom = bounds
            diagnostic_draw.rectangle((left + dx, top + dy, right + dx, bottom + dy), outline=colour)
        diagnostic_draw.text((dx, dy + 132), f"{index + 1:02d} try{row.attempt}", fill="#442a20")
    board_path = ROOT / "art/boards/portraits_34.png"
    if not board_path.exists():
        board.save(board_path, optimize=True)
    with Image.open(board_path) as frozen_board:
        assert frozen_board.convert("RGB").tobytes() == board.tobytes()
    diagnostic.save(DATA / "structural_inspection_overlay.png")
    assert board_path.stat().st_size <= 1_500_000
    payload = {"schema_version": 1, "status": "CANDIDATE_CALIBRATION_FAIL", "counts": counts,
        "calibration": {"leave_one_out_pass": 3, "leave_one_out_total": 3,
            "numeric_negative_rejections": 1, "model_negative_rejections": 1, "negative_total": 3,
            "passed": False, "limitation": "Three crops from one locked portrait are not independent source works; photo/painterly negative controls falsely pass both judges."},
        "board": {"path": str(board_path.relative_to(ROOT)), "sha256": digest(board_path), "file_bytes": board_path.stat().st_size},
        "scorecards": [{"id": row.id, "path": str(row.path), "sha256": digest(row.path),
            "references": [{"path": str(path), "sha256": digest(path)} for path in row.refs]} for row in reviews],
        "records": records}
    _ = (DOCS / "portrait_manifest.json").write_text(json.dumps(payload, indent=2) + "\n")
    print(json.dumps(counts))


if __name__ == "__main__":
    main()
