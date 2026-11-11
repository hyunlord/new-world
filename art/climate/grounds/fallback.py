"""Render explicitly existing-art season/edge demonstrations; never biome replacements."""

import json
from dataclasses import asdict

from PIL import Image

from art.climate.grounds.process import DATA, ROOT
from art.compositor.edges import Boundary, EdgeField, boundary_steps, render_tile
from art.compositor.tone import Condition, season_ground
from art.flora.normalize import digest


def main() -> None:
    out = DATA / "existing_recipe_demo"
    out.mkdir(exist_ok=True)
    sources = {
        key: ROOT / f"art/style05/env/ground/{key}.png"
        for key in ("meadow", "forest_floor", "hill_rock", "river_bank")
    }
    textures = {
        key: Image.open(path).convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS)
        for key, path in sources.items()
    }
    board = Image.new("RGBA", (256, 256))
    for y, texture in enumerate(textures.values()):
        for x, condition in enumerate(
            (Condition.DAY, Condition.DAY, Condition.DROUGHT, Condition.WINTER)
        ):
            board.paste(season_ground(texture, condition), (x * 64, y * 64))
    board.save(out / "seasons_1x.png")
    board.resize((512, 512), Image.Resampling.NEAREST).save(out / "seasons_2x.png")
    edges = []
    for band, a, b in [
        ("harsh", "forest_floor", "hill_rock"),
        ("neutral", "meadow", "forest_floor"),
        ("mild", "meadow", "river_bank"),
    ]:
        terrain = {(x, y): a if x < 2 else b for y in range(3) for x in range(4)}
        field = EdgeField(terrain, textures)
        joined = Image.new("RGBA", (256, 192))
        for cell in terrain:
            joined.paste(render_tile(field, cell), (cell[0] * 64, cell[1] * 64))
        path = out / f"edge_{band}_1x.png"
        joined.save(path)
        joined.resize((512, 384), Image.Resampling.NEAREST).save(
            out / f"edge_{band}_2x.png"
        )
        metrics = boundary_steps(joined, Boundary((128, 0), 192, "vertical"))
        assert metrics.longest_identical_step <= 24
        edges.append(
            {
                "band_demo_only": band,
                "a": a,
                "b": b,
                "board": str(path),
                "boundary": asdict(metrics),
                "seed": 5005,
                "blend_px": 12,
            }
        )
    (out / "recipes.json").write_text(
        json.dumps(
            {
                "status": "EXISTING_ART_DEMONSTRATION_NOT_NEW_BIOME_COVERAGE",
                "sources": {
                    k: {"path": str(v.relative_to(ROOT)), "sha256": digest(v)}
                    for k, v in sources.items()
                },
                "season_operations": ["DAY", "DAY", "DROUGHT", "WINTER"],
                "edges": edges,
                "biome_aliases_applied": False,
                "note": "No tundra/steppe/scrub/marsh visual replacement claimed; climate names label proposed recipe demonstrations only.",
            },
            indent=2,
        )
        + "\n"
    )


if __name__ == "__main__":
    main()
