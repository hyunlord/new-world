# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "numpy", "typer"]
# ///
# How to run: python3 -m art.compositor.polish2_walls.
"""Same-scene cap comparisons and schema-driven material-room specimens."""
from __future__ import annotations

import json
from pathlib import Path
from typing import Final, assert_never

from PIL import Image

from art.compositor.assets import REPO
from art.compositor.geometry import perimeter
from art.compositor.material_layers import (
    MaterialChoice,
    MaterialScene,
    OwnerMode,
    SceneMaterials,
)
from art.compositor.models import Overlay, Scene
from art.compositor.polish2_layout import (
    BoardLayout,
    BoardPanel,
    BoardRecord,
    make_board,
)
from art.compositor.render import RenderInputs, RenderSettings
from art.compositor.struct_render import (
    ENV1,
    SCENES,
    compose,
    save_lossless,
    struct_catalog,
)
from art.compositor.tone import Condition
from art.compositor.walls import Door, render_walls
from art.materials.render import load_textures
from art.materials.resolve import read_catalog

EXTERNAL: Final = Path("/Users/rexxa/work/artpolish2-data/boards")
CROP: Final = (0, 138, 448, 716)


def frozen_inputs() -> RenderInputs:
    """Load the existing real noon snapshot and its unchanged declared overlay."""
    return RenderInputs(Scene.read(SCENES / "scene_day6_noon.json"),
                        Overlay.read(SCENES / "m5_overlay.json"), struct_catalog(),
                        ENV1 / "reference/current_day6_noon_1x_hud.png")


def owner_materials(scene_materials: MaterialScene, mode: OwnerMode) -> MaterialScene | None:
    """Keep the frozen legacy comparison separate from schema-driven choices."""
    match mode.renderer:
        case "legacy":
            return None
        case "material":
            return MaterialScene(scene_materials.catalog, scene_materials.choices, mode.top)
    assert_never(mode.renderer)


def render_comparisons(inputs: RenderInputs, scene_materials: MaterialScene) -> tuple[tuple[str, Path], ...]:
    """Use exactly one scene, viewpoint and source set for every cap choice."""
    outputs: list[tuple[str, Path]] = []
    EXTERNAL.mkdir(parents=True, exist_ok=True)
    for mode in scene_materials.choices.owner_modes:
        selection = owner_materials(scene_materials, mode)
        product = compose(inputs, RenderSettings(1, Condition.DAY, True), materials=selection)
        path = EXTERNAL / f"walls_{mode.id}.png"
        save_lossless(product.image, path, enforce_cap=False)
        _ = path.with_suffix(".json").write_text(json.dumps(product.evidence, indent=2) + "\n")
        outputs.append((mode.label, path))
    return tuple(outputs)


def material_room(choice: MaterialChoice, scene_materials: MaterialScene) -> Path:
    """Render one identical three-by-three room through the public material API."""
    resolved = scene_materials.resolve(choice)
    textures = load_textures(REPO, resolved)
    cells = perimeter((3, 3))
    chunk = render_walls(cells, textures, (Door((1, 2), textures.door_open),))
    canvas = Image.new("RGBA", (224, 224), (242, 236, 219, 255))
    floor = struct_catalog().image("floor_packed_earth")
    floor = floor.resize((64, 64), Image.Resampling.LANCZOS)
    for y in range(3):
        for x in range(3):
            canvas.alpha_composite(floor, (16 + x * 64, 16 + y * 64))
    canvas.alpha_composite(chunk.image, (16 + chunk.origin[0], 32 + chunk.origin[1]))
    path = EXTERNAL / f"room_{choice.style_id}_{choice.material_id}.png"
    save_lossless(canvas.convert("RGB"), path, enforce_cap=False)
    _ = path.with_suffix(".json").write_text(resolved.model_dump_json(indent=2) + "\n")
    return path


def walls_board(inputs: RenderInputs, materials: MaterialScene) -> BoardRecord:
    """Preserve all native/2× views across a capped main sheet and supplements."""
    scenes = render_comparisons(inputs, materials)
    panels: list[BoardPanel] = []
    crop_width, crop_height = CROP[2] - CROP[0], CROP[3] - CROP[1]
    supplements: list[BoardRecord] = []
    detail_y = 56 + crop_height + 44
    for column, (label, path) in enumerate(scenes):
        x = 12 + column * 540
        panels.append(BoardPanel(path, CROP, (x, 56), f"{label} / full scene 1x"))
        panels.append(BoardPanel(path, (0, 138, 128, 266), (x, detail_y), "House top/corner / exact 2x", 2))
        panels.append(BoardPanel(path, (0, 458, 128, 586), (x + 264, detail_y), "Store top/corner / exact 2x", 2))
        supplement = BoardLayout((24 + crop_width * 2, 84 + crop_height * 2),
                                 f"{label} / complete matched house and storehouse / 2x",
                                 (BoardPanel(path, CROP, (12, 56), "Full 2x; same crop as main 1x", 2),))
        mode = materials.choices.owner_modes[column]
        supplements.append(make_board(REPO / f"art/boards/walls_material_strip.{mode.id}_2x.png", supplement))
    materials_y = detail_y + 300
    for column, choice in enumerate(materials.choices.samples):
        path = material_room(choice, materials)
        panels.append(BoardPanel(path, (0, 0, 224, 224), (12 + column * 232, materials_y), choice.label))
    width = max(24 + len(scenes) * 540, 24 + len(materials.choices.samples) * 232)
    layout = BoardLayout((width, materials_y + 258),
                         "WALL MATERIALS / full1x + exact2x details / full2x in named a-b-c supplements", tuple(panels))
    split = {"initial_full_board_bytes": 2145964, "after_lossless_pngcrush_bytes": 2145964,
             "file_limit_bytes": 1500000, "native_scene_crop": CROP,
             "reason": "Full three-variant 1x plus full2x plus material row exceeded the file cap. All requested full2x views survive in separate lossless PNGs; no downsampling or quantization.",
             "supplements": [record.model_dump(mode="json") for record in supplements]}
    _ = (REPO / "docs/v8/artpolish2/WALL_BOARD_SUPPLEMENTS.json").write_text(json.dumps(split, indent=2) + "\n")
    return make_board(REPO / "art/boards/walls_material_strip.png", layout)


def main() -> None:
    materials = MaterialScene(read_catalog(REPO), SceneMaterials.read())
    record = walls_board(frozen_inputs(), materials)
    print(record.model_dump_json(indent=2))


if __name__ == "__main__":
    main()
