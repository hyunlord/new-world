# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "numpy", "typer"]
# ///
# How to run: python3 -m art.compositor.polish2_boards.
"""Reproduce the three ART-POLISH2 owner boards with bound render evidence."""
from __future__ import annotations

import json
from typing import Final

from art.compositor.assets import REPO
from art.compositor.face_variants import FacePlan
from art.compositor.material_layers import MaterialScene, SceneMaterials
from art.compositor.polish2_faces import faces_board
from art.compositor.polish2_layout import BoardRecord
from art.compositor.polish2_walls import EXTERNAL, frozen_inputs, walls_board
from art.compositor.render import RenderInputs, RenderSettings, sha256
from art.compositor.struct_render import compose, save_lossless
from art.compositor.tone import Condition
from art.materials.resolve import read_catalog

DOCS: Final = REPO / "docs/v8/artpolish2"


def crowd_board(inputs: RenderInputs, plan: FacePlan) -> BoardRecord:
    """Tint each real agent before depth composition, retaining the frozen scene."""
    product = compose(inputs, RenderSettings(1, Condition.DAY, True), face_plan=plan)
    path = REPO / "art/boards/crowd_faces.png"
    path.parent.mkdir(parents=True, exist_ok=True)
    save_lossless(product.image, path)
    EXTERNAL.mkdir(parents=True, exist_ok=True)
    evidence = EXTERNAL / "crowd_faces.json"
    _ = evidence.write_text(json.dumps(product.evidence, indent=2) + "\n")
    record = BoardRecord(path=str(path.relative_to(REPO)), sha256=sha256(path),
                         bytes=path.stat().st_size, dimensions=product.image.size,
                         title="Frozen ENV1 noon scene; real-id variants; existing declared overlay", panels=())
    _ = path.with_suffix(".json").write_text(json.dumps({**record.model_dump(mode="json"),
        "render_evidence": str(evidence), "render_evidence_sha256": sha256(evidence),
        "face_plan_sha256": sha256(REPO / "art/character/FACE_VARIANTS.json"),
        "variant_transforms_sha256": sha256(REPO / "art/character/variant_transforms.json"),
        "source_scene_sha256": sha256(REPO / "art/compositor/scenes/scene_day6_noon.json"),
        "age_limitation": "Frozen scene records life stage, not numerical age; representative 10/30/50 only selects the appropriate variant pool."}, indent=2) + "\n")
    return record


def main() -> None:
    inputs, plan = frozen_inputs(), FacePlan.read()
    materials = MaterialScene(read_catalog(REPO), SceneMaterials.read())
    faces = faces_board(plan)
    records = (walls_board(inputs, materials), *faces.outputs, crowd_board(inputs, plan))
    paths = (REPO / "art/compositor/polish2_scene_materials.json", REPO / "art/character/FACE_VARIANTS.json",
             *tuple((REPO / "art/compositor").glob("polish2_*.py")),
             REPO / "art/compositor/drawing.py", REPO / "art/compositor/struct_render.py",
             REPO / "art/compositor/material_layers.py", REPO / "art/compositor/face_variants.py")
    manifest = {"boards": [record.model_dump(mode="json") for record in records],
                "faces_index": "art/boards/faces_owner.index.json",
                "faces_index_sha256": sha256(REPO / "art/boards/faces_owner.index.json"),
                "input_sha256": {str(path.relative_to(REPO)): sha256(path) for path in paths},
                "raw_directory": str(EXTERNAL), "owner_approval": "PENDING"}
    _ = (DOCS / "BOARDS.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps([{"path": record.path, "bytes": record.bytes, "sha256": record.sha256}
                      for record in records], indent=2))


if __name__ == "__main__":
    main()
