# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: uv run --with scikit-image --with pillow --with scipy python -m art.pipeline_env1.animal_evidence
"""Freeze animal calibration and create blinded whole-frame review boards."""
from __future__ import annotations

import json
import random
from dataclasses import asdict
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.pipeline2.judge import (
    Anchor,
    Calibration,
    Fold,
    ModelScores,
    bounds,
    compare,
    judge,
    measure,
)
from art.pipeline_env1.animal_normalize import (
    ANIMALS,
    DATA,
    OUT,
    ROOT,
    JsonValue,
    digest,
)

DOCS: Final = ROOT / "docs/v8/artenv1"
SCORES: Final = {
    "wolf": ModelScores(5, 5, 5, 5, "Matches the references very closely with the same inked linework, textured watercolor washes, earthy brown palette, and consistent upper-left lighting."),
    "deer": ModelScores(4, 4, 4, 5, "Overall a very close match to the reference style with correct medium, line, lighting, and palette."),
}
MODEL_STEMS: Final = {"wolf": "wolf_try3", "deer": "deer"}
FROZEN_ANCHORS: Final = (("wolf", 2), ("deer", 1))



def main() -> None:
    """Use unchanged judge v2 thresholds and keep acceptance pending blind review."""
    anchors: list[Anchor] = []
    for species, attempt in FROZEN_ANCHORS:
        path = DATA / f"{species}_try{attempt}_S_stand.png"
        anchors.append(Anchor(species + "_S_stand", "animal", str(path), digest(path), measure(path)))
    folds: list[Fold] = []
    for held in anchors:
        checks = compare(held.metrics, bounds("animal", [a for a in anchors if a.name != held.name]))
        folds.append(Fold(held.name, "animal", all(checks.values()), checks))
    calibration = Calibration(anchors, [bounds("animal", anchors)], folds)
    calibration_path = DOCS / "animal_judge_calibration.json"
    _ = calibration_path.write_text(json.dumps(asdict(calibration), indent=2) + "\n")
    controls: list[dict[str, JsonValue]] = []
    for name in ("negative_painterly", "negative_woodcut", "negative_photo_try1"):
        path = Path("/Users/rexxa/work/arttrial2-data") / f"{name}.png"
        controls.append(asdict(judge(path, calibration, family="animal")))
    model_records: list[dict[str, JsonValue]] = []
    verdicts: list[dict[str, JsonValue]] = []
    frame_paths: list[Path] = []
    board_dir = OUT / "boards/round1"
    board_dir.mkdir(parents=True, exist_ok=True)
    for animal in ANIMALS:
        species = animal.species
        raw = DATA / f"{species}_try{animal.attempt}.png"
        card = DATA / f"{MODEL_STEMS[species]}_style_scorecard.png"
        scores = SCORES[species]
        model_records.append({"species": species, "raw_sha256": digest(raw), "scores": asdict(scores),
                              "mechanism": "built-in image_gen scorecard, visually transcribed",
                              "scorecard": str(card), "scorecard_sha256": digest(card),
                              "prompt_sha256": digest(DATA / f"{MODEL_STEMS[species]}_style_judge.prompt.txt"),
                              "references": [{"path": str(p.relative_to(ROOT)), "sha256": digest(p)}
                                             for p in (ROOT / "art/style05/anchors/person.png",
                                                       ROOT / "art/style05/anchors/scene.png")]})
        frames = ("stand", "walk", "run", animal.special_pose, "dead")
        sheet = Image.new("RGBA", (480, 160))
        for row, facing in enumerate(("S", "E")):
            for column, pose in enumerate(frames):
                native = DATA / f"{species}_try{animal.attempt}_{facing}_{pose}.png"
                verdict = judge(native, calibration, scores, family="animal", model_family_verified=True)
                verdicts.append({"species": species, "facing": facing, "pose": pose, **asdict(verdict)})
                path = OUT / species / facing / f"{pose}.png"
                with Image.open(path) as frame:
                    sheet.alpha_composite(frame, (column * 96, row * 80 + (16 if pose == "dead" else 0)))
                frame_paths.append(path)
        sheet.save(board_dir / f"{species}_sheet_1x.png")
        sheet.resize((960, 320), Image.Resampling.NEAREST).save(board_dir / f"{species}_sheet_2x.png")
    key_records: list[dict[str, JsonValue]] = []
    for scale in (1, 2):
        order = list(frame_paths)
        random.Random(20261024 + scale).shuffle(order)
        cell_w, cell_h = 144 * scale, 110 * scale
        board = Image.new("RGB", (cell_w * 5, cell_h * 4), (48, 47, 42))
        draw = ImageDraw.Draw(board)
        for index, path in enumerate(order):
            x, y = index % 5 * cell_w, index // 5 * cell_h
            identifier = f"R1-{scale}X-{index + 1:02d}"
            draw.text((x + 7, y + 6), identifier, fill=(236, 228, 209))
            with Image.open(path) as frame:
                resized = frame.resize((frame.width * scale, frame.height * scale), Image.Resampling.NEAREST)
                board.paste(resized, (x + (cell_w - resized.width) // 2, y + 23 * scale), resized)
            key_records.append({"id": identifier, "scale": scale, "species": path.parts[-3],
                                "facing": path.parts[-2], "pose": path.stem,
                                "image": str(path.relative_to(ROOT)), "image_sha256": digest(path)})
        board.save(board_dir / f"blind_frames_{scale}x.png")
    _ = (DATA / "review_answer_key_round1.json").write_text(json.dumps(key_records, indent=2) + "\n")
    result = {"calibration_sha256": digest(calibration_path), "leave_one_out_pass": all(f.passed for f in folds),
              "negative_controls": controls, "negative_controls_pass": all(not c["numeric_pass"] for c in controls),
              "model_reviews": model_records, "native_panel_verdicts": verdicts,
              "numeric_pass_panels": sum(v["numeric_pass"] is True for v in verdicts),
              "style_pass_panels": sum(v["passed"] is True for v in verdicts), "panel_count": len(verdicts),
              "semantic_review": "Round1 PENDING: two NEW fresh no-key reviewers at 1x and 2x; round0 retained",
              "calibration_limit": "Frozen wolf try2 and deer try1 native S-stand panels; wolf try3 did not change the calibration. Each anchor held out against the other. Sheet model judges establish their compatibility with locked originals, not owner adoption."}
    _ = (DOCS / "animal_style_evidence.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"loo": result["leave_one_out_pass"], "negative_controls": result["negative_controls_pass"],
                      "numeric": result["numeric_pass_panels"], "style": result["style_pass_panels"],
                      "boards": str(board_dir), "answer_key": str(DATA / "review_answer_key_round1.json")}))


if __name__ == "__main__":
    main()
