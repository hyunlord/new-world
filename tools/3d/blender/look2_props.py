"""Rebuild L2 stations/yard props and refresh their canonical manifest rows.

blender -b --factory-startup -P tools/3d/blender/look2_props.py
Run asset_items.py and asset_buildings.py afterwards, then merge_assets.py.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import asset_stations as A
import common as C
import items as I
import look2_props_shapes as H
import look2_props_yard as Y
import stations as S
import stations_w3d as W


def build() -> None:
    """Keep historical IDs stable, append yard props, and export all station states."""
    C.reset_scene()
    entries = S.build({})
    I.build(entries)
    for identity, (builder, footprint) in Y.PROPS.items():
        m = C.Model(identity)
        builder(m)
        H.finish(m)
        C.save_static(m, footprint, entries, {"family": "prop", "art_pass": "look2"})
    for identity, row in entries.items():
        assert row["triangles"] <= 6500, (identity, row["triangles"])
        assert row["height"] > 0, identity
        assert set(row["materials"]) <= set(C.PALETTE), identity
    path = Path(C.MODELS_DIR) / "look2_props_manifest.json"
    path.write_text(json.dumps({"models": entries}, indent=2, sort_keys=True) + "\n")
    C.update_manifest(entries)
    C.reset_scene()
    A.build()
    C.reset_scene()
    W.build()
    print(f"LOOK2_PROPS {len(entries)} replacement/yard models; 63 asset pieces; 13 W3D states")


if __name__ == "__main__":
    build()
