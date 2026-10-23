# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "typer"]
# ///
# How to run: python3 -m art.compositor.polish_boards.
"""Reproducible POLISH wall comparisons with fixed pixels and frozen scene IDs."""
from __future__ import annotations

import json
from pathlib import Path
from typing import Final

from art.compositor.assets import REPO
from art.compositor.models import Overlay, Scene
from art.compositor.render import RenderInputs, RenderSettings, sha256
from art.compositor.struct_boards import BOARDS, Panel, detail_demos, make_board
from art.compositor.struct_render import (
    ENV1,
    SCENES,
    compose,
    save_lossless,
    struct_catalog,
)
from art.compositor.tone import Condition
from art.compositor.wall_style import CapStyle, WallParameters

EXTERNAL: Final = Path("/Users/rexxa/work/artpolish-data/walls")
DOCS: Final = REPO / "docs/v8/artpolish"


def render_variant(inputs: RenderInputs, name: str, style: CapStyle, params: WallParameters) -> Path:
    destination = EXTERNAL / name
    destination.mkdir(parents=True, exist_ok=True)
    output = destination / "scene_day6_noon_day_1x_figures.png"
    product = compose(inputs, RenderSettings(1, Condition.DAY, True), cap_style=style, wall_params=params)
    save_lossless(product.image, output, enforce_cap=False)
    _ = output.with_suffix(".json").write_text(json.dumps(product.evidence, indent=2) + "\n")
    return output


def main() -> None:
    """Render original and dark wall styles at one fixed snapshot and viewport."""
    EXTERNAL.mkdir(parents=True, exist_ok=True)
    params = WallParameters.read()
    scene = Scene.read(SCENES / "scene_day6_noon.json")
    overlay = Overlay.read(SCENES / "m5_overlay.json")
    inputs = RenderInputs(scene, overlay, struct_catalog(), ENV1 / "reference/current_day6_noon_1x_hud.png")
    original = render_variant(inputs, "struct", "struct", params)
    default = render_variant(inputs, "dark_045", "dark", params)
    crop = (0, 140, 960, 740)
    detail_crop = (0, 140, 448, 500)
    make_board(BOARDS / "polish_walls_owner.png", (
        Panel("STRUCT cap / same detail / 1x", original, detail_crop),
        Panel("Dark cap V x 0.45 / same detail / 1x", default, detail_crop),
        Panel("STRUCT cap / same detail / 2x", original, detail_crop, 2),
        Panel("Dark cap V x 0.45 / same detail / 2x", default, detail_crop, 2),
    ))
    full_paths: list[Path] = []
    for name, source in (("struct", original), ("dark", default)):
        for scale in (1, 2):
            destination = BOARDS / f"polish_walls_owner.{name}_{scale}x.png"
            make_board(destination, (Panel(f"{name.upper()} full STRUCT board 1 crop / {scale}x", source, crop, scale),))
            full_paths.append(destination)
    house, workshop, hut = detail_demos(external=default.parent, docs=EXTERNAL, wall_params=params)
    make_board(BOARDS / "polish_closeups_2x.png", (
        Panel("House / declared occupants / 2x", house, (0, 130, 448, 490), 2),
        Panel("Stone store / 2x", default, (0, 450, 320, 730), 2),
        Panel("Dug pocket / 2x", default, (448, 130, 640, 354), 2),
        Panel("River + ford / 2x", default, (544, 330, 736, 458), 2),
        Panel("Workshop / declared workers / 2x", workshop, (608, 130, 928, 386), 2),
        Panel("3x3 hut / declared fixture / 2x", hut, (0, 0, 224, 224), 2),
    ))
    variants: list[Panel] = []
    for value in params.owner_value_choices:
        selected = params.model_copy(update={"value_multiplier": value})
        source = default if value == params.value_multiplier else render_variant(inputs, f"dark_{round(value * 100):03}", "dark", selected)
        variants.append(Panel(f"Owner choice / cap V x {value:.2f} / 2x", source, (0, 140, 448, 500), 2))
    make_board(BOARDS / "polish_cap_darkness_strip.png", tuple(variants))
    paths = (*tuple(BOARDS / name for name in ("polish_walls_owner.png", "polish_closeups_2x.png", "polish_cap_darkness_strip.png")), *full_paths)
    records = [{"path": str(path.relative_to(REPO)), "sha256": sha256(path), "bytes": path.stat().st_size} for path in paths]
    sources = [Path(__file__), Path(__file__).with_name("wall_params.json"),
               *[Path(__file__).with_name(name) for name in ("walls.py", "wall_style.py", "edges.py", "struct_ground.py", "struct_layers.py", "struct_render.py", "struct_boards.py")]]
    manifest = {"boards": records, "source_sha256": {str(path.relative_to(REPO)): sha256(path) for path in sources},
                "owner_approval": "PENDING", "full_scene_crop": list(crop), "same_detail_crop": list(detail_crop), "scales": [1, 2],
                "params": params.model_dump(mode="json"), "raw_render_directory": str(EXTERNAL),
                "comparison": "Both styles use the same corrected dug-floor topology and blend exclusion; STRUCT retains its original cap, end and corner textures.",
                "layout_reason": "Full scene x two styles x two scales in one PNG was 1850232 bytes after pngcrush -brute. Main board preserves matched detail crops at both scales; four supplementary boards preserve the entire original scene crop at both scales without downsampling or quantization."}
    _ = (DOCS / "WALL_BOARDS.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(records, indent=2))


if __name__ == "__main__":
    main()
