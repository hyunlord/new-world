# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: uv run --with pillow --with numpy --with scikit-image python -m art.pipeline_env1.built_autotile
"""Derive wall connectivity from generated strips and whole L-shaped corners."""
from __future__ import annotations

import json
from dataclasses import asdict
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image, ImageDraw

from art.pipeline2.autotile import Sources, recipe
from art.pipeline2.autotile_v2 import compose
from art.pipeline_env1.built_process import DOC, OUT, digest
from art.pipeline_env1.items_records import JsonRecord, JsonValue

CORNERS: Final = {"log": (0, 515, 800, 1315), "stone": (-200, 300, 1000, 1500)}


def main() -> None:
    manifest_path = DOC / "built_assets.json"
    manifest = JsonRecord.read(manifest_path)
    asset_rows = [row for row in manifest.records("assets") if not row.text("id").rsplit("_", 1)[-1].isdigit()]
    by_id = {row.text("id"): row for row in asset_rows}
    seams: list[dict[str, JsonValue]] = []
    flagged_pairs = 0
    topology: list[dict[str, JsonValue]] = []
    topology_pass_count = 0
    board = Image.new("RGBA", (1024, 768), (78, 76, 64, 255))
    draw = ImageDraw.Draw(board)
    for material_index, material in enumerate(("log", "stone")):
        straight_row = by_id[f"wall_{material}_straight"]
        corner_row = by_id[f"wall_{material}_corner"]
        with Image.open(straight_row.text("raw")) as raw:
            original = raw.convert("RGBA")
        ys, xs = np.nonzero(np.asarray(original)[:, :, 3] >= 128)
        if not len(xs):
            raise RuntimeError(f"Empty straight alpha: {straight_row.text('raw')}")
        bounds = (int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1)
        strip = original.crop((0, bounds[1], original.width, bounds[3])).resize((64, 16), Image.Resampling.LANCZOS)
        straight = Image.new("RGBA", (64, 64))
        straight.alpha_composite(strip, (0, 24))
        with Image.open(corner_row.text("raw")) as raw:
            corner = raw.convert("RGBA").crop(CORNERS[material]).resize((64, 64), Image.Resampling.LANCZOS)
        sources = Sources(straight, corner, strip.crop((24, 0, 40, 16)))
        tiles = [compose(mask, sources) for mask in range(16)]
        for mask, tile in enumerate(tiles):
            rgba: NDArray[np.uint8] = np.asarray(tile)
            alpha = rgba[:, :, 3]
            edges = (alpha[0], alpha[:, -1], alpha[-1], alpha[:, 0])
            observed = sum(bit for bit, edge in zip((1, 2, 4, 8), edges, strict=True) if int(edge.max()) >= 128)
            topology_pass_count += observed == mask
            topology.append({"material": material, "mask": mask, "observed": observed, "pass": observed == mask})
            name = f"wall_{material}_{mask:02d}"
            path = OUT / f"{name}.png"
            tile.save(path, optimize=True)
            row: dict[str, JsonValue] = {"id": name, "path": str(path), "width": 64, "height": 64, "footprint": [1, 1],
                   "family": "buildings", "attempts": 0, "status": "DERIVED_CANDIDATE",
                   "sha256": digest(path), "recipe": asdict(recipe(mask)), "bits": {"N": 1, "E": 2, "S": 4, "W": 8},
                   "source_ids": [straight_row.text("id"), corner_row.text("id")],
                   "source_sha256": [straight_row.text("raw_sha256"), corner_row.text("raw_sha256")],
                   "straight_bounds": [0, bounds[1], original.width, bounds[3]],
                   "corner_crop": CORNERS[material], "bytes": path.stat().st_size,
                   "operations": ["generated straight alpha crop then64x16 resize", "generated2x2 elbow crop then64x64 resize",
                                  "quarter rotations; generated-arm overlays for terminals/T/X; no mirror-blend"],
                   "limitation": "Quarter rotations rotate baked lighting; junction appearance requires visual review."}
            asset_rows.append(JsonRecord(row))
            _ = path.with_suffix(".json").write_text(json.dumps(row, indent=2) + "\n")
            x, y = mask % 8 * 128, material_index * 384 + mask // 8 * 120
            board.alpha_composite(tile, (x + 32, y + 24))
            draw.text((x + 20, y + 96), f"{material} {mask:02d}", fill="white")
        for axis, outbound, inbound in (("E-W", 2, 8), ("S-N", 4, 1)):
            for first in range(16):
                for second in range(16):
                    if first & outbound and second & inbound:
                        left: NDArray[np.uint8] = np.asarray(tiles[first])
                        right: NDArray[np.uint8] = np.asarray(tiles[second])
                        edge_a, edge_b = (left[:, -1], right[:, 0]) if outbound == 2 else (left[-1], right[0])
                        visible = (edge_a[:, 3] >= 128) | (edge_b[:, 3] >= 128)
                        rgb_a = edge_a[:, :3].astype(float) * edge_a[:, 3:4] / 255
                        rgb_b = edge_b[:, :3].astype(float) * edge_b[:, 3:4] / 255
                        error = float(np.abs(rgb_a - rgb_b)[visible].mean())
                        alpha_mismatch: NDArray[np.bool_] = np.not_equal(edge_a[:, 3] >= 128, edge_b[:, 3] >= 128)
                        alpha_gap = int(np.count_nonzero(alpha_mismatch))
                        flagged_pairs += error > 18 or alpha_gap > 2
                        seams.append({"material": material, "pair": [first, second], "axis": axis,
                                      "visible_rgb_mae": error, "alpha_gap_pixels": alpha_gap,
                                      "flagged": error > 18 or alpha_gap > 2})
        for pair_index, (first, second) in enumerate(((10, 10), (6, 10), (10, 12), (7, 13))):
            x, y = pair_index * 256, material_index * 384 + 250
            pair = Image.new("RGBA", (128, 64))
            pair.alpha_composite(tiles[first], (0, 0))
            pair.alpha_composite(tiles[second], (64, 0))
            board.alpha_composite(pair.resize((256, 128), Image.Resampling.NEAREST), (x, y))
    board.convert("RGB").save(OUT / "autotile_board.png", optimize=True)
    manifest.fields["assets"] = [row.fields for row in asset_rows]
    _ = manifest_path.write_text(json.dumps(manifest.fields, indent=2) + "\n")
    _ = (DOC / "built_autotile.json").write_text(json.dumps({"topology": topology, "compatible_pairs": seams,
                                                        "all_topology_pass": topology_pass_count == len(topology)}, indent=2) + "\n")
    print(json.dumps({"topology_pass": topology_pass_count, "total": len(topology),
                      "flagged_pairs": flagged_pairs, "pairs": len(seams)}))


if __name__ == "__main__":
    main()
